use std::time::Duration;

use rand::Rng;
use reqwest::StatusCode;

use ferriskey_webhook::entities::retry_policy::RetryPolicy;
use ferriskey_webhook::entities::webhook_delivery::DeliveryErrorCode;

pub const MAX_ATTEMPTS: u32 = RetryPolicy::SYSTEM_DEFAULT.max_attempts();
pub const MAX_TOTAL_DELAY: Duration = RetryPolicy::SYSTEM_DEFAULT.max_total_delay();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Transport,
    Status(StatusCode),
}

impl DeliveryOutcome {
    pub fn error_code(self) -> DeliveryErrorCode {
        match self {
            Self::Transport => DeliveryErrorCode::Transport,
            Self::Status(status) => DeliveryErrorCode::HttpStatus(status.as_u16()),
        }
    }
}

pub fn is_retryable(outcome: DeliveryOutcome) -> bool {
    outcome.error_code().is_retryable()
}

pub fn backoff_delay(attempt: u32) -> Duration {
    RetryPolicy::SYSTEM_DEFAULT.backoff_delay(attempt)
}

pub fn apply_jitter(base: Duration, rng: &mut impl Rng) -> Duration {
    let millis = u64::try_from(base.as_millis()).unwrap_or(u64::MAX);
    Duration::from_millis(rng.gen_range(0..=millis))
}

pub fn should_retry(attempt: u32, outcome: DeliveryOutcome, elapsed_delay: Duration) -> bool {
    attempt < MAX_ATTEMPTS && elapsed_delay < MAX_TOTAL_DELAY && is_retryable(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_transport_failure_maps_to_the_domain_transport_code() {
        assert_eq!(
            DeliveryOutcome::Transport.error_code(),
            DeliveryErrorCode::Transport
        );
    }

    #[test]
    fn a_status_maps_to_the_domain_code_carrying_the_same_number() {
        assert_eq!(
            DeliveryOutcome::Status(StatusCode::SERVICE_UNAVAILABLE).error_code(),
            DeliveryErrorCode::HttpStatus(503)
        );
        assert_eq!(
            DeliveryOutcome::Status(StatusCode::TOO_MANY_REQUESTS).error_code(),
            DeliveryErrorCode::HttpStatus(429)
        );
    }

    #[test]
    fn transport_errors_are_always_retryable() {
        assert!(is_retryable(DeliveryOutcome::Transport));
    }

    #[test]
    fn server_errors_and_rate_limits_are_retryable() {
        for status in [
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::REQUEST_TIMEOUT,
        ] {
            assert!(
                is_retryable(DeliveryOutcome::Status(status)),
                "{status} should be retryable"
            );
        }
    }

    #[test]
    fn ordinary_client_errors_are_not_retryable() {
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::UNPROCESSABLE_ENTITY,
        ] {
            assert!(
                !is_retryable(DeliveryOutcome::Status(status)),
                "{status} should not be retryable"
            );
        }
    }

    #[test]
    fn redirect_and_success_statuses_are_not_retryable() {
        assert!(!is_retryable(DeliveryOutcome::Status(StatusCode::FOUND)));
        assert!(!is_retryable(DeliveryOutcome::Status(StatusCode::OK)));
    }

    #[test]
    fn the_re_exported_budget_still_matches_the_system_default() {
        assert_eq!(MAX_ATTEMPTS, 5);
        assert_eq!(MAX_TOTAL_DELAY, Duration::from_secs(120));
    }

    #[test]
    fn should_retry_stops_once_the_attempt_cap_is_reached() {
        let outcome = DeliveryOutcome::Transport;

        assert!(should_retry(MAX_ATTEMPTS - 1, outcome, Duration::ZERO));
        assert!(!should_retry(MAX_ATTEMPTS, outcome, Duration::ZERO));
        assert!(!should_retry(MAX_ATTEMPTS + 1, outcome, Duration::ZERO));
    }

    #[test]
    fn should_retry_stops_once_the_total_delay_cap_is_reached() {
        let outcome = DeliveryOutcome::Transport;

        assert!(should_retry(
            1,
            outcome,
            MAX_TOTAL_DELAY - Duration::from_millis(1)
        ));
        assert!(!should_retry(1, outcome, MAX_TOTAL_DELAY));
        assert!(!should_retry(
            1,
            outcome,
            MAX_TOTAL_DELAY + Duration::from_secs(1)
        ));
    }

    #[test]
    fn should_retry_never_retries_a_non_retryable_status() {
        let outcome = DeliveryOutcome::Status(StatusCode::NOT_FOUND);

        assert!(!should_retry(1, outcome, Duration::ZERO));
    }
}
