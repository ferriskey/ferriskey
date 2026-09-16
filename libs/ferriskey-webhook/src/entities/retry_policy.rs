use std::num::NonZeroU32;
use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MIN_ATTEMPTS: u32 = 1;
pub const MAX_ATTEMPTS: u32 = 20;
pub const MIN_BASE_DELAY_MS: u32 = 100;
pub const MAX_BASE_DELAY_MS: u32 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RetryPolicyError {
    #[error("max_attempts must be between {MIN_ATTEMPTS} and {MAX_ATTEMPTS}, got {got}")]
    AttemptsOutOfRange { got: u32 },
    #[error("base_delay_ms must be between {MIN_BASE_DELAY_MS} and {MAX_BASE_DELAY_MS}, got {got}")]
    BaseDelayOutOfRange { got: u32 },
    #[error("max_delay_ms ({max_delay_ms}) must be at least base_delay_ms ({base_delay_ms})")]
    MaxDelayBelowBaseDelay {
        base_delay_ms: u32,
        max_delay_ms: u32,
    },
    #[error(
        "max_total_delay_ms ({max_total_delay_ms}) must be at least max_delay_ms ({max_delay_ms})"
    )]
    MaxTotalDelayBelowMaxDelay {
        max_delay_ms: u32,
        max_total_delay_ms: u32,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicyOverride {
    pub max_attempts: Option<u32>,
    pub base_delay_ms: Option<u32>,
    pub max_delay_ms: Option<u32>,
    pub max_total_delay_ms: Option<u32>,
}

impl RetryPolicyOverride {
    pub fn is_empty(&self) -> bool {
        self.max_attempts.is_none()
            && self.base_delay_ms.is_none()
            && self.max_delay_ms.is_none()
            && self.max_total_delay_ms.is_none()
    }

    fn or(self, fallback: Self) -> Self {
        Self {
            max_attempts: self.max_attempts.or(fallback.max_attempts),
            base_delay_ms: self.base_delay_ms.or(fallback.base_delay_ms),
            max_delay_ms: self.max_delay_ms.or(fallback.max_delay_ms),
            max_total_delay_ms: self.max_total_delay_ms.or(fallback.max_total_delay_ms),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: NonZeroU32,
    base_delay: Duration,
    max_delay: Duration,
    max_total_delay: Duration,
}

const fn non_zero(value: u32) -> NonZeroU32 {
    match NonZeroU32::new(value) {
        Some(value) => value,
        None => NonZeroU32::MIN,
    }
}

impl RetryPolicy {
    pub const SYSTEM_DEFAULT: Self = Self {
        max_attempts: non_zero(5),
        base_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(30),
        max_total_delay: Duration::from_secs(120),
    };

    pub const fn max_attempts(&self) -> u32 {
        self.max_attempts.get()
    }

    pub const fn base_delay(&self) -> Duration {
        self.base_delay
    }

    pub const fn max_delay(&self) -> Duration {
        self.max_delay
    }

    pub const fn max_total_delay(&self) -> Duration {
        self.max_total_delay
    }

    pub fn as_override(&self) -> RetryPolicyOverride {
        RetryPolicyOverride {
            max_attempts: Some(self.max_attempts.get()),
            base_delay_ms: Some(as_millis(self.base_delay)),
            max_delay_ms: Some(as_millis(self.max_delay)),
            max_total_delay_ms: Some(as_millis(self.max_total_delay)),
        }
    }

    pub fn resolve(
        webhook: RetryPolicyOverride,
        realm: RetryPolicyOverride,
    ) -> Result<Self, RetryPolicyError> {
        Self::try_from(webhook.or(realm))
    }

    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        let exponent = attempt.saturating_sub(1);
        let multiplier = 1u64.checked_shl(exponent).unwrap_or(u64::MAX);
        let base = u64::try_from(self.base_delay.as_millis()).unwrap_or(u64::MAX);

        Duration::from_millis(base.saturating_mul(multiplier)).min(self.max_delay)
    }

    pub fn next_attempt_delay(
        &self,
        attempt: u32,
        elapsed: Duration,
        rng: &mut impl Rng,
    ) -> Option<Duration> {
        if attempt >= self.max_attempts.get() || elapsed >= self.max_total_delay {
            return None;
        }

        let remaining = self.max_total_delay.saturating_sub(elapsed);
        let ceiling = self.backoff_delay(attempt + 1).min(remaining);
        let ceiling = u64::try_from(ceiling.as_millis()).unwrap_or(u64::MAX);

        Some(Duration::from_millis(rng.gen_range(0..=ceiling)))
    }
}

fn as_millis(value: Duration) -> u32 {
    u32::try_from(value.as_millis()).unwrap_or(u32::MAX)
}

impl TryFrom<RetryPolicyOverride> for RetryPolicy {
    type Error = RetryPolicyError;

    fn try_from(value: RetryPolicyOverride) -> Result<Self, Self::Error> {
        let fallback = Self::SYSTEM_DEFAULT;

        let requested_attempts = value.max_attempts.unwrap_or(fallback.max_attempts.get());
        let max_attempts = NonZeroU32::new(requested_attempts)
            .filter(|attempts| attempts.get() <= MAX_ATTEMPTS)
            .ok_or(RetryPolicyError::AttemptsOutOfRange {
                got: requested_attempts,
            })?;

        let base_delay_ms = value
            .base_delay_ms
            .unwrap_or(as_millis(fallback.base_delay));
        if !(MIN_BASE_DELAY_MS..=MAX_BASE_DELAY_MS).contains(&base_delay_ms) {
            return Err(RetryPolicyError::BaseDelayOutOfRange { got: base_delay_ms });
        }

        let max_delay_ms = value.max_delay_ms.unwrap_or(as_millis(fallback.max_delay));
        if max_delay_ms < base_delay_ms {
            return Err(RetryPolicyError::MaxDelayBelowBaseDelay {
                base_delay_ms,
                max_delay_ms,
            });
        }

        let max_total_delay_ms = value
            .max_total_delay_ms
            .unwrap_or(as_millis(fallback.max_total_delay));
        if max_total_delay_ms < max_delay_ms {
            return Err(RetryPolicyError::MaxTotalDelayBelowMaxDelay {
                max_delay_ms,
                max_total_delay_ms,
            });
        }

        Ok(Self {
            max_attempts,
            base_delay: Duration::from_millis(base_delay_ms.into()),
            max_delay: Duration::from_millis(max_delay_ms.into()),
            max_total_delay: Duration::from_millis(max_total_delay_ms.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    const DEFAULT: RetryPolicy = RetryPolicy::SYSTEM_DEFAULT;

    fn full_override(
        max_attempts: u32,
        base_delay_ms: u32,
        max_delay_ms: u32,
        max_total_delay_ms: u32,
    ) -> RetryPolicyOverride {
        RetryPolicyOverride {
            max_attempts: Some(max_attempts),
            base_delay_ms: Some(base_delay_ms),
            max_delay_ms: Some(max_delay_ms),
            max_total_delay_ms: Some(max_total_delay_ms),
        }
    }

    #[test]
    fn system_default_matches_the_behaviour_shipped_before_the_outbox() {
        assert_eq!(DEFAULT.max_attempts(), 5);
        assert_eq!(DEFAULT.base_delay(), Duration::from_millis(500));
        assert_eq!(DEFAULT.max_delay(), Duration::from_secs(30));
        assert_eq!(DEFAULT.max_total_delay(), Duration::from_secs(120));
    }

    #[test]
    fn resolving_without_any_override_yields_the_system_default() {
        let resolved = RetryPolicy::resolve(
            RetryPolicyOverride::default(),
            RetryPolicyOverride::default(),
        )
        .expect("the system default is valid");

        assert_eq!(resolved, DEFAULT);
    }

    #[test]
    fn a_realm_override_applies_when_the_webhook_sets_nothing() {
        let realm = full_override(9, 1_000, 40_000, 300_000);

        let resolved = RetryPolicy::resolve(RetryPolicyOverride::default(), realm)
            .expect("the realm override is within bounds");

        assert_eq!(resolved.max_attempts(), 9);
        assert_eq!(resolved.base_delay(), Duration::from_millis(1_000));
    }

    #[test]
    fn a_webhook_override_wins_over_the_realm() {
        let webhook = full_override(3, 200, 1_000, 5_000);
        let realm = full_override(9, 1_000, 40_000, 300_000);

        let resolved =
            RetryPolicy::resolve(webhook, realm).expect("both overrides are within bounds");

        assert_eq!(resolved.max_attempts(), 3);
        assert_eq!(resolved.base_delay(), Duration::from_millis(200));
    }

    #[test]
    fn a_partial_webhook_override_inherits_every_field_it_omits() {
        let webhook = RetryPolicyOverride {
            max_attempts: Some(10),
            ..RetryPolicyOverride::default()
        };
        let realm = RetryPolicyOverride {
            base_delay_ms: Some(2_000),
            ..RetryPolicyOverride::default()
        };

        let resolved = RetryPolicy::resolve(webhook, realm).expect("both overrides are valid");

        assert_eq!(resolved.max_attempts(), 10);
        assert_eq!(resolved.base_delay(), Duration::from_millis(2_000));
        assert_eq!(resolved.max_delay(), DEFAULT.max_delay());
        assert_eq!(resolved.max_total_delay(), DEFAULT.max_total_delay());
    }

    #[test]
    fn zero_attempts_is_rejected() {
        let error = RetryPolicy::try_from(full_override(0, 500, 30_000, 120_000))
            .expect_err("zero attempts must not produce a policy");

        assert_eq!(error, RetryPolicyError::AttemptsOutOfRange { got: 0 });
    }

    #[test]
    fn more_attempts_than_the_cap_is_rejected() {
        let error = RetryPolicy::try_from(full_override(21, 500, 30_000, 120_000))
            .expect_err("21 attempts exceeds the cap");

        assert_eq!(error, RetryPolicyError::AttemptsOutOfRange { got: 21 });
    }

    #[test]
    fn the_attempt_bounds_themselves_are_accepted() {
        assert!(RetryPolicy::try_from(full_override(MIN_ATTEMPTS, 500, 30_000, 120_000)).is_ok());
        assert!(RetryPolicy::try_from(full_override(MAX_ATTEMPTS, 500, 30_000, 120_000)).is_ok());
    }

    #[test]
    fn a_base_delay_below_the_floor_is_rejected() {
        let error = RetryPolicy::try_from(full_override(5, 50, 30_000, 120_000))
            .expect_err("50ms is below the floor");

        assert_eq!(error, RetryPolicyError::BaseDelayOutOfRange { got: 50 });
    }

    #[test]
    fn a_base_delay_above_the_ceiling_is_rejected() {
        let error = RetryPolicy::try_from(full_override(5, 60_001, 120_000, 300_000))
            .expect_err("60001ms is above the ceiling");

        assert_eq!(error, RetryPolicyError::BaseDelayOutOfRange { got: 60_001 });
    }

    #[test]
    fn a_max_delay_below_the_base_delay_is_rejected() {
        let error = RetryPolicy::try_from(full_override(5, 5_000, 1_000, 120_000))
            .expect_err("a cap below the base delay is incoherent");

        assert_eq!(
            error,
            RetryPolicyError::MaxDelayBelowBaseDelay {
                base_delay_ms: 5_000,
                max_delay_ms: 1_000,
            }
        );
    }

    #[test]
    fn a_total_budget_below_a_single_delay_is_rejected() {
        let error = RetryPolicy::try_from(full_override(5, 500, 30_000, 10_000))
            .expect_err("a budget smaller than one delay can never elapse normally");

        assert_eq!(
            error,
            RetryPolicyError::MaxTotalDelayBelowMaxDelay {
                max_delay_ms: 30_000,
                max_total_delay_ms: 10_000,
            }
        );
    }

    #[test]
    fn backoff_doubles_until_it_reaches_the_single_delay_cap() {
        assert_eq!(DEFAULT.backoff_delay(1), Duration::from_millis(500));
        assert_eq!(DEFAULT.backoff_delay(2), Duration::from_millis(1_000));
        assert_eq!(DEFAULT.backoff_delay(3), Duration::from_millis(2_000));
        assert_eq!(DEFAULT.backoff_delay(4), Duration::from_millis(4_000));
        assert_eq!(DEFAULT.backoff_delay(5), Duration::from_millis(8_000));
        assert_eq!(DEFAULT.backoff_delay(8), DEFAULT.max_delay());
        assert_eq!(DEFAULT.backoff_delay(1_000), DEFAULT.max_delay());
    }

    #[test]
    fn backoff_never_overflows_on_an_absurd_attempt_number() {
        assert_eq!(DEFAULT.backoff_delay(u32::MAX), DEFAULT.max_delay());
    }

    #[test]
    fn jitter_stays_within_zero_and_the_uncapped_backoff() {
        let mut rng = StdRng::seed_from_u64(42);

        for attempt in 0..DEFAULT.max_attempts() {
            let ceiling = DEFAULT.backoff_delay(attempt + 1);
            for _ in 0..200 {
                let delay = DEFAULT
                    .next_attempt_delay(attempt, Duration::ZERO, &mut rng)
                    .expect("the budget is untouched");

                assert!(delay <= ceiling, "{delay:?} exceeded {ceiling:?}");
            }
        }
    }

    #[test]
    fn jitter_is_not_collapsed_to_a_constant() {
        let mut rng = StdRng::seed_from_u64(7);
        let mut seen = std::collections::HashSet::new();

        for _ in 0..200 {
            let delay = DEFAULT
                .next_attempt_delay(4, Duration::ZERO, &mut rng)
                .expect("the budget is untouched");
            seen.insert(delay.as_millis());
        }

        assert!(seen.len() > 1, "full jitter must spread across a range");
    }

    #[test]
    fn the_budget_closes_once_the_attempt_cap_is_reached() {
        let mut rng = StdRng::seed_from_u64(1);

        assert!(
            DEFAULT
                .next_attempt_delay(DEFAULT.max_attempts() - 1, Duration::ZERO, &mut rng)
                .is_some()
        );
        assert!(
            DEFAULT
                .next_attempt_delay(DEFAULT.max_attempts(), Duration::ZERO, &mut rng)
                .is_none()
        );
        assert!(
            DEFAULT
                .next_attempt_delay(DEFAULT.max_attempts() + 1, Duration::ZERO, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn the_budget_closes_once_the_total_delay_is_spent() {
        let mut rng = StdRng::seed_from_u64(2);
        let total = DEFAULT.max_total_delay();

        assert!(
            DEFAULT
                .next_attempt_delay(1, total - Duration::from_millis(1), &mut rng)
                .is_some()
        );
        assert!(DEFAULT.next_attempt_delay(1, total, &mut rng).is_none());
        assert!(
            DEFAULT
                .next_attempt_delay(1, total + Duration::from_secs(1), &mut rng)
                .is_none()
        );
    }

    #[test]
    fn a_late_attempt_never_sleeps_past_the_remaining_budget() {
        let mut rng = StdRng::seed_from_u64(9);
        let remaining = Duration::from_millis(50);
        let elapsed = DEFAULT.max_total_delay() - remaining;

        for _ in 0..500 {
            let delay = DEFAULT
                .next_attempt_delay(4, elapsed, &mut rng)
                .expect("the budget is not spent yet");

            assert!(delay <= remaining, "{delay:?} overran {remaining:?}");
        }
    }

    #[test]
    fn a_policy_survives_a_round_trip_through_its_override() {
        let policy = RetryPolicy::try_from(full_override(7, 250, 12_000, 90_000))
            .expect("the override is within bounds");

        let round_tripped = RetryPolicy::try_from(policy.as_override())
            .expect("a policy's own override is valid by construction");

        assert_eq!(round_tripped, policy);
    }

    #[test]
    fn an_empty_override_is_reported_as_empty() {
        assert!(RetryPolicyOverride::default().is_empty());
        assert!(
            !RetryPolicyOverride {
                max_attempts: Some(3),
                ..RetryPolicyOverride::default()
            }
            .is_empty()
        );
    }
}
