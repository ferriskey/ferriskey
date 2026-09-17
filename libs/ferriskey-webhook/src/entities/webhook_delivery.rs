use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ferriskey_domain::generate_uuid_v7;
use ferriskey_domain::realm::RealmId;

use crate::entities::webhook_trigger::WebhookTrigger;

pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MAX_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WebhookDeliveryId(Uuid);

impl WebhookDeliveryId {
    pub fn new() -> Self {
        Self(generate_uuid_v7())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for WebhookDeliveryId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WebhookDeliveryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Pending,
    Delivering,
    Succeeded,
    Failed,
}

impl DeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivering => "delivering",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "delivering" => Some(Self::Delivering),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryErrorCode {
    ReservedHeader,
    MalformedEndpoint,
    MissingHost,
    DnsResolutionFailed,
    NoUsableAddress,
    CleartextNotAllowed,
    ClientBuildFailed,
    HeaderEncodingFailed,
    Transport,
    HttpStatus(u16),
}

impl DeliveryErrorCode {
    pub fn as_code(self) -> String {
        match self {
            Self::ReservedHeader => "reserved_header".to_string(),
            Self::MalformedEndpoint => "malformed_endpoint".to_string(),
            Self::MissingHost => "missing_host".to_string(),
            Self::DnsResolutionFailed => "dns_resolution_failed".to_string(),
            Self::NoUsableAddress => "no_usable_address".to_string(),
            Self::CleartextNotAllowed => "cleartext_not_allowed".to_string(),
            Self::ClientBuildFailed => "client_build_failed".to_string(),
            Self::HeaderEncodingFailed => "header_encoding_failed".to_string(),
            Self::Transport => "transport_error".to_string(),
            Self::HttpStatus(status) => format!("http_{status}"),
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "reserved_header" => Some(Self::ReservedHeader),
            "malformed_endpoint" => Some(Self::MalformedEndpoint),
            "missing_host" => Some(Self::MissingHost),
            "dns_resolution_failed" => Some(Self::DnsResolutionFailed),
            "no_usable_address" => Some(Self::NoUsableAddress),
            "cleartext_not_allowed" => Some(Self::CleartextNotAllowed),
            "client_build_failed" => Some(Self::ClientBuildFailed),
            "header_encoding_failed" => Some(Self::HeaderEncodingFailed),
            "transport_error" => Some(Self::Transport),
            other => other
                .strip_prefix("http_")
                .and_then(|status| status.parse().ok())
                .map(Self::HttpStatus),
        }
    }

    pub fn is_retryable(self) -> bool {
        match self {
            Self::ReservedHeader
            | Self::MalformedEndpoint
            | Self::MissingHost
            | Self::NoUsableAddress
            | Self::CleartextNotAllowed => false,
            Self::DnsResolutionFailed
            | Self::ClientBuildFailed
            | Self::HeaderEncodingFailed
            | Self::Transport => true,
            Self::HttpStatus(status) => {
                (500..600).contains(&status) || status == 429 || status == 408
            }
        }
    }

    pub fn status_code(self) -> Option<u16> {
        match self {
            Self::HttpStatus(status) => Some(status),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Succeeded {
        at: DateTime<Utc>,
        status_code: u16,
    },
    Retrying {
        at: DateTime<Utc>,
        next_attempt_at: DateTime<Utc>,
        error: DeliveryErrorCode,
        detail: Option<String>,
    },
    Exhausted {
        at: DateTime<Utc>,
        error: DeliveryErrorCode,
        detail: Option<String>,
    },
}

impl DeliveryOutcome {
    pub fn status(&self) -> DeliveryStatus {
        match self {
            Self::Succeeded { .. } => DeliveryStatus::Succeeded,
            Self::Retrying { .. } => DeliveryStatus::Pending,
            Self::Exhausted { .. } => DeliveryStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: WebhookDeliveryId,
    pub realm_id: RealmId,
    pub webhook_id: Uuid,
    pub event: WebhookTrigger,
    pub resource_id: Uuid,
    pub payload: serde_json::Value,
    pub status: DeliveryStatus,
    pub attempt_count: u32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub leased_until: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_status_code: Option<u16>,
    pub last_error_code: Option<DeliveryErrorCode>,
    pub last_error_detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WebhookDelivery {
    pub fn pending(
        realm_id: RealmId,
        webhook_id: Uuid,
        event: WebhookTrigger,
        resource_id: Uuid,
        payload: serde_json::Value,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: WebhookDeliveryId::new(),
            realm_id,
            webhook_id,
            event,
            resource_id,
            payload,
            status: DeliveryStatus::Pending,
            attempt_count: 0,
            next_attempt_at: Some(now),
            leased_until: None,
            last_attempt_at: None,
            last_status_code: None,
            last_error_code: None,
            last_error_detail: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply(&mut self, outcome: DeliveryOutcome) {
        self.status = outcome.status();
        self.attempt_count = self.attempt_count.saturating_add(1);
        self.leased_until = None;

        match outcome {
            DeliveryOutcome::Succeeded { at, status_code } => {
                self.next_attempt_at = None;
                self.last_attempt_at = Some(at);
                self.last_status_code = Some(status_code);
                self.last_error_code = None;
                self.last_error_detail = None;
                self.updated_at = at;
            }
            DeliveryOutcome::Retrying {
                at,
                next_attempt_at,
                error,
                detail,
            } => {
                self.next_attempt_at = Some(next_attempt_at);
                self.last_attempt_at = Some(at);
                self.last_status_code = error.status_code();
                self.last_error_code = Some(error);
                self.last_error_detail = detail;
                self.updated_at = at;
            }
            DeliveryOutcome::Exhausted { at, error, detail } => {
                self.next_attempt_at = None;
                self.last_attempt_at = Some(at);
                self.last_status_code = error.status_code();
                self.last_error_code = Some(error);
                self.last_error_detail = detail;
                self.updated_at = at;
            }
        }
    }

    pub fn requeue(&mut self, now: DateTime<Utc>) {
        self.status = DeliveryStatus::Pending;
        self.attempt_count = 0;
        self.next_attempt_at = Some(now);
        self.leased_until = None;
        self.updated_at = now;
    }

    pub fn elapsed_since_created(&self, now: DateTime<Utc>) -> Duration {
        (now - self.created_at).to_std().unwrap_or(Duration::ZERO)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryFilter {
    pub status: Option<DeliveryStatus>,
    pub event: Option<WebhookTrigger>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DeliveryFilterError {
    #[error("limit must be between 1 and {MAX_PAGE_SIZE}, got {got}")]
    LimitOutOfRange { got: u32 },
}

impl DeliveryFilter {
    pub fn new(
        status: Option<DeliveryStatus>,
        event: Option<WebhookTrigger>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Self, DeliveryFilterError> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(DeliveryFilterError::LimitOutOfRange { got: limit });
        }

        Ok(Self {
            status,
            event,
            limit,
            offset: offset.unwrap_or(0),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryPage {
    pub items: Vec<WebhookDelivery>,
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("a fixed epoch second is representable")
    }

    fn sample() -> WebhookDelivery {
        WebhookDelivery::pending(
            RealmId::new(Uuid::nil()),
            Uuid::nil(),
            WebhookTrigger::UserCreated,
            Uuid::nil(),
            serde_json::json!({"event": "user.created"}),
            now(),
        )
    }

    #[test]
    fn a_new_delivery_is_pending_and_immediately_due() {
        let delivery = sample();

        assert_eq!(delivery.status, DeliveryStatus::Pending);
        assert_eq!(delivery.attempt_count, 0);
        assert_eq!(delivery.next_attempt_at, Some(now()));
        assert!(delivery.leased_until.is_none());
        assert!(delivery.last_error_code.is_none());
    }

    #[test]
    fn error_codes_keep_the_exact_strings_already_written_to_the_database() {
        assert_eq!(
            DeliveryErrorCode::ReservedHeader.as_code(),
            "reserved_header"
        );
        assert_eq!(
            DeliveryErrorCode::MalformedEndpoint.as_code(),
            "malformed_endpoint"
        );
        assert_eq!(DeliveryErrorCode::MissingHost.as_code(), "missing_host");
        assert_eq!(
            DeliveryErrorCode::DnsResolutionFailed.as_code(),
            "dns_resolution_failed"
        );
        assert_eq!(
            DeliveryErrorCode::NoUsableAddress.as_code(),
            "no_usable_address"
        );
        assert_eq!(
            DeliveryErrorCode::ClientBuildFailed.as_code(),
            "client_build_failed"
        );
        assert_eq!(
            DeliveryErrorCode::HeaderEncodingFailed.as_code(),
            "header_encoding_failed"
        );
        assert_eq!(DeliveryErrorCode::Transport.as_code(), "transport_error");
        assert_eq!(DeliveryErrorCode::HttpStatus(503).as_code(), "http_503");
    }

    #[test]
    fn every_error_code_survives_a_round_trip_through_its_string() {
        let codes = [
            DeliveryErrorCode::ReservedHeader,
            DeliveryErrorCode::MalformedEndpoint,
            DeliveryErrorCode::MissingHost,
            DeliveryErrorCode::DnsResolutionFailed,
            DeliveryErrorCode::NoUsableAddress,
            DeliveryErrorCode::CleartextNotAllowed,
            DeliveryErrorCode::ClientBuildFailed,
            DeliveryErrorCode::HeaderEncodingFailed,
            DeliveryErrorCode::Transport,
            DeliveryErrorCode::HttpStatus(500),
            DeliveryErrorCode::HttpStatus(429),
        ];

        for code in codes {
            assert_eq!(DeliveryErrorCode::parse(&code.as_code()), Some(code));
        }
    }

    #[test]
    fn an_unknown_error_string_parses_to_nothing_rather_than_a_wrong_variant() {
        assert_eq!(DeliveryErrorCode::parse("teapot"), None);
        assert_eq!(DeliveryErrorCode::parse("http_"), None);
        assert_eq!(DeliveryErrorCode::parse("http_abc"), None);
        assert_eq!(DeliveryErrorCode::parse(""), None);
    }

    #[test]
    fn transport_class_failures_are_retryable() {
        for code in [
            DeliveryErrorCode::Transport,
            DeliveryErrorCode::DnsResolutionFailed,
            DeliveryErrorCode::ClientBuildFailed,
            DeliveryErrorCode::HeaderEncodingFailed,
        ] {
            assert!(code.is_retryable(), "{code:?} should be retryable");
        }
    }

    #[test]
    fn configuration_failures_are_never_retryable() {
        for code in [
            DeliveryErrorCode::ReservedHeader,
            DeliveryErrorCode::MalformedEndpoint,
            DeliveryErrorCode::MissingHost,
            DeliveryErrorCode::NoUsableAddress,
            DeliveryErrorCode::CleartextNotAllowed,
        ] {
            assert!(!code.is_retryable(), "{code:?} should not be retryable");
        }
    }

    #[test]
    fn server_errors_rate_limits_and_request_timeouts_are_retryable() {
        for status in [500, 502, 503, 504, 599, 429, 408] {
            assert!(
                DeliveryErrorCode::HttpStatus(status).is_retryable(),
                "{status} should be retryable"
            );
        }
    }

    #[test]
    fn ordinary_client_errors_redirects_and_successes_are_not_retryable() {
        for status in [400, 401, 403, 404, 422, 302, 200] {
            assert!(
                !DeliveryErrorCode::HttpStatus(status).is_retryable(),
                "{status} should not be retryable"
            );
        }
    }

    #[test]
    fn a_successful_outcome_closes_the_delivery() {
        let mut delivery = sample();

        delivery.apply(DeliveryOutcome::Succeeded {
            at: now(),
            status_code: 204,
        });

        assert_eq!(delivery.status, DeliveryStatus::Succeeded);
        assert_eq!(delivery.attempt_count, 1);
        assert_eq!(delivery.next_attempt_at, None);
        assert_eq!(delivery.last_status_code, Some(204));
        assert_eq!(delivery.last_error_code, None);
        assert_eq!(delivery.last_error_detail, None);
        assert!(delivery.leased_until.is_none());
    }

    #[test]
    fn a_retrying_outcome_reopens_the_delivery_at_its_next_due_date() {
        let mut delivery = sample();
        let due = now() + chrono::Duration::seconds(30);

        delivery.apply(DeliveryOutcome::Retrying {
            at: now(),
            next_attempt_at: due,
            error: DeliveryErrorCode::HttpStatus(503),
            detail: Some("service unavailable".to_string()),
        });

        assert_eq!(delivery.status, DeliveryStatus::Pending);
        assert_eq!(delivery.attempt_count, 1);
        assert_eq!(delivery.next_attempt_at, Some(due));
        assert_eq!(delivery.last_status_code, Some(503));
        assert_eq!(
            delivery.last_error_code,
            Some(DeliveryErrorCode::HttpStatus(503))
        );
        assert!(delivery.leased_until.is_none());
    }

    #[test]
    fn an_exhausted_outcome_is_terminal_and_keeps_the_last_error() {
        let mut delivery = sample();

        delivery.apply(DeliveryOutcome::Exhausted {
            at: now(),
            error: DeliveryErrorCode::Transport,
            detail: Some("connection refused".to_string()),
        });

        assert_eq!(delivery.status, DeliveryStatus::Failed);
        assert_eq!(delivery.next_attempt_at, None);
        assert_eq!(delivery.last_error_code, Some(DeliveryErrorCode::Transport));
        assert_eq!(
            delivery.last_error_detail,
            Some("connection refused".to_string())
        );
        assert!(delivery.status.is_terminal());
    }

    #[test]
    fn a_non_http_failure_records_no_status_code() {
        let mut delivery = sample();

        delivery.apply(DeliveryOutcome::Exhausted {
            at: now(),
            error: DeliveryErrorCode::DnsResolutionFailed,
            detail: None,
        });

        assert_eq!(delivery.last_status_code, None);
    }

    #[test]
    fn attempts_accumulate_across_successive_outcomes() {
        let mut delivery = sample();

        for expected in 1..=3 {
            delivery.apply(DeliveryOutcome::Retrying {
                at: now(),
                next_attempt_at: now() + chrono::Duration::seconds(1),
                error: DeliveryErrorCode::Transport,
                detail: None,
            });
            assert_eq!(delivery.attempt_count, expected);
        }
    }

    #[test]
    fn requeueing_a_failed_delivery_resets_its_attempt_budget() {
        let mut delivery = sample();
        delivery.apply(DeliveryOutcome::Exhausted {
            at: now(),
            error: DeliveryErrorCode::Transport,
            detail: Some("gone".to_string()),
        });

        let replay_at = now() + chrono::Duration::hours(1);
        delivery.requeue(replay_at);

        assert_eq!(delivery.status, DeliveryStatus::Pending);
        assert_eq!(delivery.attempt_count, 0);
        assert_eq!(delivery.next_attempt_at, Some(replay_at));
        assert!(delivery.leased_until.is_none());
    }

    #[test]
    fn the_elapsed_budget_is_measured_from_creation() {
        let delivery = sample();

        assert_eq!(
            delivery.elapsed_since_created(now() + chrono::Duration::seconds(90)),
            Duration::from_secs(90)
        );
        assert_eq!(delivery.elapsed_since_created(now()), Duration::ZERO);
    }

    #[test]
    fn a_clock_running_backwards_yields_no_elapsed_budget_rather_than_panicking() {
        let delivery = sample();

        assert_eq!(
            delivery.elapsed_since_created(now() - chrono::Duration::hours(1)),
            Duration::ZERO
        );
    }

    #[test]
    fn only_succeeded_and_failed_are_terminal() {
        assert!(DeliveryStatus::Succeeded.is_terminal());
        assert!(DeliveryStatus::Failed.is_terminal());
        assert!(!DeliveryStatus::Pending.is_terminal());
        assert!(!DeliveryStatus::Delivering.is_terminal());
    }

    #[test]
    fn every_status_survives_a_round_trip_through_its_string() {
        for status in [
            DeliveryStatus::Pending,
            DeliveryStatus::Delivering,
            DeliveryStatus::Succeeded,
            DeliveryStatus::Failed,
        ] {
            assert_eq!(DeliveryStatus::parse(status.as_str()), Some(status));
            assert!(status.as_str().len() <= 16);
        }

        assert_eq!(DeliveryStatus::parse("exploded"), None);
    }

    #[test]
    fn an_outcome_announces_the_status_it_will_produce() {
        let succeeded = DeliveryOutcome::Succeeded {
            at: now(),
            status_code: 200,
        };
        let retrying = DeliveryOutcome::Retrying {
            at: now(),
            next_attempt_at: now(),
            error: DeliveryErrorCode::Transport,
            detail: None,
        };
        let exhausted = DeliveryOutcome::Exhausted {
            at: now(),
            error: DeliveryErrorCode::Transport,
            detail: None,
        };

        assert_eq!(succeeded.status(), DeliveryStatus::Succeeded);
        assert_eq!(retrying.status(), DeliveryStatus::Pending);
        assert_eq!(exhausted.status(), DeliveryStatus::Failed);
    }

    #[test]
    fn a_filter_defaults_its_page_size_when_none_is_given() {
        let filter = DeliveryFilter::new(None, None, None, None).expect("no bounds are crossed");

        assert_eq!(filter.limit, DEFAULT_PAGE_SIZE);
        assert_eq!(filter.offset, 0);
    }

    #[test]
    fn a_page_size_above_the_cap_is_rejected_rather_than_clamped() {
        let error = DeliveryFilter::new(None, None, Some(MAX_PAGE_SIZE + 1), None)
            .expect_err("an oversized page must be refused");

        assert_eq!(
            error,
            DeliveryFilterError::LimitOutOfRange {
                got: MAX_PAGE_SIZE + 1
            }
        );
    }

    #[test]
    fn a_zero_page_size_is_rejected() {
        assert_eq!(
            DeliveryFilter::new(None, None, Some(0), None),
            Err(DeliveryFilterError::LimitOutOfRange { got: 0 })
        );
    }

    #[test]
    fn the_page_size_cap_itself_is_accepted() {
        let filter = DeliveryFilter::new(None, None, Some(MAX_PAGE_SIZE), None)
            .expect("the cap is a legal page size");

        assert_eq!(filter.limit, MAX_PAGE_SIZE);
    }
}
