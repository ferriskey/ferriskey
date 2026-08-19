use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url, redirect};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::sync::{Semaphore, mpsc};
use tokio::time::sleep;
use tracing::error;
use uuid::Uuid;

use ferriskey_webhook::endpoint::{is_forbidden_address, reject_reserved_headers};
use ferriskey_webhook::signing::{DELIVERY_HEADER, SIGNATURE_HEADER, TIMESTAMP_HEADER, sign};

use crate::domain::common::generate_uuid_v7;
use crate::entity::webhooks::{
    ActiveModel as WebhookActiveModel, Column as WebhookColumn, Entity as WebhookEntity,
};

use super::retry::{self, DeliveryOutcome};

const QUEUE_CAPACITY: usize = 1024;
const MAX_CONCURRENT_DELIVERIES: usize = 16;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct DeliveryJob {
    pub webhook_id: Uuid,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub secret: String,
    pub body: Arc<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryFailure {
    ReservedHeader,
    MalformedEndpoint,
    MissingHost,
    DnsResolutionFailed,
    NoUsableAddress,
    ClientBuildFailed,
    HeaderEncodingFailed,
    Transport,
    Status(StatusCode),
}

impl DeliveryFailure {
    fn code(self) -> String {
        match self {
            Self::ReservedHeader => "reserved_header".to_string(),
            Self::MalformedEndpoint => "malformed_endpoint".to_string(),
            Self::MissingHost => "missing_host".to_string(),
            Self::DnsResolutionFailed => "dns_resolution_failed".to_string(),
            Self::NoUsableAddress => "no_usable_address".to_string(),
            Self::ClientBuildFailed => "client_build_failed".to_string(),
            Self::HeaderEncodingFailed => "header_encoding_failed".to_string(),
            Self::Transport => "transport_error".to_string(),
            Self::Status(status) => format!("http_{}", status.as_u16()),
        }
    }

    /// `ReservedHeader`, `MalformedEndpoint`, `MissingHost` and `NoUsableAddress` describe the
    /// webhook's own configuration rather than a transient failure of its endpoint: retrying
    /// leaves the same rejection in place, and for `NoUsableAddress` specifically, retrying a
    /// request that was just correctly refused as a possible SSRF target is the wrong instinct
    /// even if a future resolution would come back public again.
    fn outcome(self) -> Option<DeliveryOutcome> {
        match self {
            Self::ReservedHeader
            | Self::MalformedEndpoint
            | Self::MissingHost
            | Self::NoUsableAddress => None,
            Self::DnsResolutionFailed | Self::ClientBuildFailed | Self::HeaderEncodingFailed => {
                Some(DeliveryOutcome::Transport)
            }
            Self::Transport => Some(DeliveryOutcome::Transport),
            Self::Status(status) => Some(DeliveryOutcome::Status(status)),
        }
    }
}

pub fn spawn_dispatcher(db: DatabaseConnection) -> mpsc::Sender<DeliveryJob> {
    let (sender, mut receiver) = mpsc::channel::<DeliveryJob>(QUEUE_CAPACITY);

    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DELIVERIES));

        while let Some(job) = receiver.recv().await {
            let permit = match Arc::clone(&semaphore).acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => break,
            };

            let db = db.clone();
            tokio::spawn(async move {
                deliver_with_retry(job, &db).await;
                drop(permit);
            });
        }
    });

    sender
}

async fn deliver_with_retry(job: DeliveryJob, db: &DatabaseConnection) {
    if let Err(err) = reject_reserved_headers(&job.headers) {
        error!(
            webhook_id = %job.webhook_id,
            error = %err,
            "refusing webhook delivery: a configured header collides with a reserved name"
        );
        persist_outcome(
            db,
            job.webhook_id,
            Some(DeliveryFailure::ReservedHeader.code()),
        )
        .await;
        return;
    }

    let delivery_id = generate_uuid_v7();
    let mut attempt: u32 = 1;
    let mut cumulative_delay = Duration::ZERO;

    loop {
        match attempt_delivery(&job, delivery_id).await {
            Ok(()) => {
                persist_outcome(db, job.webhook_id, None).await;
                return;
            }
            Err(failure) => {
                let code = failure.code();
                let retry_eligible = match failure.outcome() {
                    Some(outcome) => retry::should_retry(attempt, outcome, cumulative_delay),
                    None => false,
                };

                if !retry_eligible {
                    error!(
                        webhook_id = %job.webhook_id,
                        attempt,
                        reason = %code,
                        "webhook delivery failed permanently"
                    );
                    persist_outcome(db, job.webhook_id, Some(code)).await;
                    return;
                }

                let delay =
                    retry::apply_jitter(retry::backoff_delay(attempt), &mut rand::thread_rng());
                cumulative_delay += delay;
                error!(
                    webhook_id = %job.webhook_id,
                    attempt,
                    reason = %code,
                    delay_ms = delay.as_millis() as u64,
                    "webhook delivery failed, retrying with backoff"
                );
                sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

/// Resolves the endpoint's host over DNS and pins the outgoing connection to the resolved
/// address that passed [`is_forbidden_address`]. Resolution happens again on every attempt,
/// including retries: a name that answered with a public address a minute ago can answer with a
/// private one now, and only the address actually dialed protects against that.
async fn attempt_delivery(job: &DeliveryJob, delivery_id: Uuid) -> Result<(), DeliveryFailure> {
    let url = Url::parse(&job.endpoint).map_err(|_| DeliveryFailure::MalformedEndpoint)?;
    let host = url
        .host_str()
        .ok_or(DeliveryFailure::MissingHost)?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or(DeliveryFailure::MissingHost)?;

    let mut resolved = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|_| DeliveryFailure::DnsResolutionFailed)?;

    let addr = resolved
        .find(|candidate| !is_forbidden_address(candidate.ip()))
        .ok_or(DeliveryFailure::NoUsableAddress)?;

    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(redirect::Policy::none())
        .resolve(&host, addr)
        .build()
        .map_err(|_| DeliveryFailure::ClientBuildFailed)?;

    let mut headers = HeaderMap::new();
    for (key, value) in &job.headers {
        match (HeaderName::from_str(key), HeaderValue::from_str(value)) {
            (Ok(name), Ok(val)) => {
                headers.insert(name, val);
            }
            (Err(e), _) => {
                error!(webhook_id = %job.webhook_id, key = %key, error = %e, "invalid webhook header name");
            }
            (_, Err(e)) => {
                error!(webhook_id = %job.webhook_id, key = %key, error = %e, "invalid webhook header value");
            }
        }
    }
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let timestamp = Utc::now().timestamp();
    let signature = sign(&job.secret, timestamp, &job.body);

    headers.insert(
        HeaderName::from_static(TIMESTAMP_HEADER),
        HeaderValue::from_str(&timestamp.to_string())
            .map_err(|_| DeliveryFailure::HeaderEncodingFailed)?,
    );
    headers.insert(
        HeaderName::from_static(DELIVERY_HEADER),
        HeaderValue::from_str(&delivery_id.to_string())
            .map_err(|_| DeliveryFailure::HeaderEncodingFailed)?,
    );
    headers.insert(
        HeaderName::from_static(SIGNATURE_HEADER),
        HeaderValue::from_str(&signature).map_err(|_| DeliveryFailure::HeaderEncodingFailed)?,
    );

    let response = client
        .post(url)
        .headers(headers)
        .body(job.body.as_ref().clone())
        .send()
        .await
        .map_err(|_| DeliveryFailure::Transport)?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(DeliveryFailure::Status(response.status()))
    }
}

/// Records the terminal outcome of a delivery job: `error_code` is `None` for a success and
/// `Some(reason)` — one of [`DeliveryFailure::code`]'s values — once retries are exhausted or the
/// failure was never retryable to begin with.
async fn persist_outcome(db: &DatabaseConnection, webhook_id: Uuid, error_code: Option<String>) {
    let now = Utc::now().naive_utc();
    let status = if error_code.is_none() {
        "success"
    } else {
        "failed"
    };

    let result = WebhookEntity::update_many()
        .set(WebhookActiveModel {
            triggered_at: Set(Some(now)),
            last_delivery_status: Set(Some(status.to_string())),
            last_delivery_error: Set(error_code),
            ..Default::default()
        })
        .filter(WebhookColumn::Id.eq(webhook_id))
        .exec(db)
        .await;

    if let Err(err) = result {
        error!(
            webhook_id = %webhook_id,
            error = %err,
            "failed to persist webhook delivery outcome"
        );
    }
}
