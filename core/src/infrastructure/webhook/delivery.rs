use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url, redirect};
use tokio::sync::{Semaphore, mpsc};
use tracing::error;
use uuid::Uuid;

use ferriskey_domain::realm::RealmId;
use ferriskey_seawatch::entities::{EventStatus, SecurityEvent, SecurityEventType};
use ferriskey_seawatch::ports::SecurityEventRepository;
use ferriskey_webhook::endpoint::{
    PrivateEndpoints, allows_cleartext, is_forbidden_address, reject_reserved_headers,
};
use ferriskey_webhook::signing::{DELIVERY_HEADER, SIGNATURE_HEADER, TIMESTAMP_HEADER, sign};

use crate::domain::webhook::entities::retry_policy::RetryPolicy;
use crate::domain::webhook::entities::webhook_delivery::{
    DeliveryErrorCode, DeliveryOutcome, WebhookDeliveryId,
};
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookDeliveryRepository;

const QUEUE_CAPACITY: usize = 1024;
const MAX_CONCURRENT_DELIVERIES: usize = 16;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct DeliveryJob {
    pub delivery_id: WebhookDeliveryId,
    pub realm_id: RealmId,
    pub webhook_id: Uuid,
    pub event: WebhookTrigger,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub secret: String,
    pub body: Arc<Vec<u8>>,
    pub attempt_count: u32,
    pub elapsed: Duration,
    pub policy: RetryPolicy,
    pub private_endpoints: PrivateEndpoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryFailure {
    ReservedHeader,
    MalformedEndpoint,
    MissingHost,
    DnsResolutionFailed,
    NoUsableAddress,
    CleartextNotAllowed,
    ClientBuildFailed,
    HeaderEncodingFailed,
    Transport,
    Status(StatusCode),
}

impl DeliveryFailure {
    fn error_code(self) -> DeliveryErrorCode {
        match self {
            Self::ReservedHeader => DeliveryErrorCode::ReservedHeader,
            Self::MalformedEndpoint => DeliveryErrorCode::MalformedEndpoint,
            Self::MissingHost => DeliveryErrorCode::MissingHost,
            Self::DnsResolutionFailed => DeliveryErrorCode::DnsResolutionFailed,
            Self::NoUsableAddress => DeliveryErrorCode::NoUsableAddress,
            Self::CleartextNotAllowed => DeliveryErrorCode::CleartextNotAllowed,
            Self::ClientBuildFailed => DeliveryErrorCode::ClientBuildFailed,
            Self::HeaderEncodingFailed => DeliveryErrorCode::HeaderEncodingFailed,
            Self::Transport => DeliveryErrorCode::Transport,
            Self::Status(status) => DeliveryErrorCode::HttpStatus(status.as_u16()),
        }
    }
}

pub fn spawn_dispatcher<D, S>(deliveries: D, security_events: S) -> mpsc::Sender<DeliveryJob>
where
    D: WebhookDeliveryRepository + Clone + 'static,
    S: SecurityEventRepository + Clone + 'static,
{
    let (sender, mut receiver) = mpsc::channel::<DeliveryJob>(QUEUE_CAPACITY);

    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DELIVERIES));

        while let Some(job) = receiver.recv().await {
            let permit = match Arc::clone(&semaphore).acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => break,
            };

            let deliveries = deliveries.clone();
            let security_events = security_events.clone();
            tokio::spawn(async move {
                deliver_once(job, &deliveries, &security_events).await;
                drop(permit);
            });
        }
    });

    sender
}

pub async fn deliver_once<D, S>(job: DeliveryJob, deliveries: &D, security_events: &S)
where
    D: WebhookDeliveryRepository,
    S: SecurityEventRepository,
{
    let at = Utc::now();

    let failure = match attempt_delivery(&job).await {
        Ok(status_code) => {
            record(
                deliveries,
                &job,
                DeliveryOutcome::Succeeded { at, status_code },
            )
            .await;
            return;
        }
        Err(failure) => failure,
    };

    let error = failure.error_code();
    let attempt_count = job.attempt_count.saturating_add(1);

    let next_delay = if error.is_retryable() {
        job.policy
            .next_attempt_delay(attempt_count, job.elapsed, &mut rand::thread_rng())
            .and_then(|delay| chrono::Duration::from_std(delay).ok())
    } else {
        None
    };

    match next_delay {
        Some(delay) => {
            error!(
                webhook_id = %job.webhook_id,
                delivery_id = %job.delivery_id,
                attempt = attempt_count,
                reason = %error.as_code(),
                delay_ms = delay.num_milliseconds(),
                "webhook delivery failed, scheduling another attempt"
            );
            record(
                deliveries,
                &job,
                DeliveryOutcome::Retrying {
                    at,
                    next_attempt_at: at + delay,
                    error,
                    detail: None,
                },
            )
            .await;
        }
        None => {
            error!(
                webhook_id = %job.webhook_id,
                delivery_id = %job.delivery_id,
                attempt = attempt_count,
                reason = %error.as_code(),
                "webhook delivery exhausted its attempts"
            );
            record(
                deliveries,
                &job,
                DeliveryOutcome::Exhausted {
                    at,
                    error,
                    detail: None,
                },
            )
            .await;
            emit_exhausted_event(security_events, &job, error).await;
        }
    }
}

async fn record<D>(deliveries: &D, job: &DeliveryJob, outcome: DeliveryOutcome)
where
    D: WebhookDeliveryRepository,
{
    if let Err(err) = deliveries.record_outcome(job.delivery_id, outcome).await {
        error!(
            webhook_id = %job.webhook_id,
            delivery_id = %job.delivery_id,
            error = ?err,
            "failed to persist webhook delivery outcome"
        );
    }
}

async fn emit_exhausted_event<S>(security_events: &S, job: &DeliveryJob, error: DeliveryErrorCode)
where
    S: SecurityEventRepository,
{
    let event = SecurityEvent::without_actor(
        job.realm_id,
        SecurityEventType::WebhookDeliveryExhausted,
        EventStatus::Failure,
    )
    .with_target("webhook".to_string(), job.webhook_id, None)
    .with_details(serde_json::json!({
        "delivery_id": job.delivery_id.as_uuid(),
        "event": job.event.to_string(),
        "error_code": error.as_code(),
        "attempts": job.attempt_count.saturating_add(1),
    }));

    if let Err(err) = security_events.store_event(event).await {
        error!(
            webhook_id = %job.webhook_id,
            delivery_id = %job.delivery_id,
            error = ?err,
            "failed to record the webhook delivery exhaustion event"
        );
    }
}

async fn attempt_delivery(job: &DeliveryJob) -> Result<u16, DeliveryFailure> {
    reject_reserved_headers(&job.headers).map_err(|_| DeliveryFailure::ReservedHeader)?;

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
        .find(|candidate| !is_forbidden_address(candidate.ip(), job.private_endpoints))
        .ok_or(DeliveryFailure::NoUsableAddress)?;

    if url.scheme() != "https" && !allows_cleartext(addr.ip(), job.private_endpoints) {
        return Err(DeliveryFailure::CleartextNotAllowed);
    }

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
                error!(webhook_id = %job.webhook_id, key = %key, error = %e, "invalid webhook header name")
            }
            (_, Err(e)) => {
                error!(webhook_id = %job.webhook_id, key = %key, error = %e, "invalid webhook header value")
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
        HeaderValue::from_str(&job.delivery_id.to_string())
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

    let status = response.status();
    if status.is_success() {
        Ok(status.as_u16())
    } else {
        Err(DeliveryFailure::Status(status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failure_maps_to_its_persisted_error_code() {
        let cases = [
            (DeliveryFailure::ReservedHeader, "reserved_header"),
            (DeliveryFailure::MalformedEndpoint, "malformed_endpoint"),
            (DeliveryFailure::MissingHost, "missing_host"),
            (
                DeliveryFailure::DnsResolutionFailed,
                "dns_resolution_failed",
            ),
            (DeliveryFailure::NoUsableAddress, "no_usable_address"),
            (DeliveryFailure::ClientBuildFailed, "client_build_failed"),
            (
                DeliveryFailure::HeaderEncodingFailed,
                "header_encoding_failed",
            ),
            (DeliveryFailure::Transport, "transport_error"),
            (
                DeliveryFailure::Status(StatusCode::SERVICE_UNAVAILABLE),
                "http_503",
            ),
        ];

        for (failure, expected) in cases {
            assert_eq!(failure.error_code().as_code(), expected);
        }
    }

    #[test]
    fn configuration_failures_stay_non_retryable_across_the_boundary() {
        for failure in [
            DeliveryFailure::ReservedHeader,
            DeliveryFailure::MalformedEndpoint,
            DeliveryFailure::MissingHost,
            DeliveryFailure::NoUsableAddress,
        ] {
            assert!(!failure.error_code().is_retryable(), "{failure:?}");
        }
    }

    #[test]
    fn transport_and_server_failures_stay_retryable_across_the_boundary() {
        for failure in [
            DeliveryFailure::Transport,
            DeliveryFailure::DnsResolutionFailed,
            DeliveryFailure::ClientBuildFailed,
            DeliveryFailure::HeaderEncodingFailed,
            DeliveryFailure::Status(StatusCode::INTERNAL_SERVER_ERROR),
            DeliveryFailure::Status(StatusCode::TOO_MANY_REQUESTS),
        ] {
            assert!(failure.error_code().is_retryable(), "{failure:?}");
        }
    }
}
