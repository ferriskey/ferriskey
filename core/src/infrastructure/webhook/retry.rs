use std::time::Duration;

use rand::Rng;
use reqwest::StatusCode;

pub const MAX_ATTEMPTS: u32 = 5;
pub const MAX_TOTAL_DELAY: Duration = Duration::from_secs(120);

const BASE_DELAY: Duration = Duration::from_millis(500);
const MAX_SINGLE_DELAY: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Transport,
    Status(StatusCode),
}

pub fn is_retryable(outcome: DeliveryOutcome) -> bool {
    match outcome {
        DeliveryOutcome::Transport => true,
        DeliveryOutcome::Status(status) => {
            status.is_server_error()
                || status == StatusCode::TOO_MANY_REQUESTS
                || status == StatusCode::REQUEST_TIMEOUT
        }
    }
}

pub fn backoff_delay(attempt: u32) -> Duration {
    let exponent = attempt.saturating_sub(1);
    let multiplier = 1u64.checked_shl(exponent).unwrap_or(u64::MAX);
    let millis = (BASE_DELAY.as_millis() as u64).saturating_mul(multiplier);
    Duration::from_millis(millis).min(MAX_SINGLE_DELAY)
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
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn backoff_schedule_doubles_up_to_the_single_delay_cap() {
        assert_eq!(backoff_delay(1), Duration::from_millis(500));
        assert_eq!(backoff_delay(2), Duration::from_millis(1_000));
        assert_eq!(backoff_delay(3), Duration::from_millis(2_000));
        assert_eq!(backoff_delay(4), Duration::from_millis(4_000));
        assert_eq!(backoff_delay(5), Duration::from_millis(8_000));
        assert_eq!(backoff_delay(8), MAX_SINGLE_DELAY);
        assert_eq!(backoff_delay(1_000), MAX_SINGLE_DELAY);
    }

    #[test]
    fn jitter_never_exceeds_or_negates_the_base_delay() {
        let mut rng = StdRng::seed_from_u64(42);
        for attempt in 1..=10u32 {
            let base = backoff_delay(attempt);
            for _ in 0..200 {
                let jittered = apply_jitter(base, &mut rng);
                assert!(
                    jittered <= base,
                    "jitter {jittered:?} exceeded base {base:?}"
                );
            }
        }
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
