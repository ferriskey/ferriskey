use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entities::{FlowStatus, StepStatus};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowFilter {
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: Option<String>,
    pub status: Option<FlowStatus>,
    pub ip_address: Option<String>,
    pub identified: Option<bool>,
    pub completed: Option<bool>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FlowSortField {
    Status,
    StartedAt,
    DurationMs,
    #[default]
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlowStats {
    pub total: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub pending_count: i64,
    pub avg_duration_ms: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DailyActivityStatsFilter {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyActivityStats {
    pub date: String,
    pub signups: i64,
    pub logins: i64,
    pub login_failures: i64,
    pub pending_logins: i64,
    pub expired_logins: i64,
    pub total_flows: i64,
    pub unique_login_users: i64,
    pub avg_login_duration_ms: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub(crate) status: StepStatus,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) details: Option<serde_json::Value>,
}

impl StepOutcome {
    pub fn success() -> Self {
        Self {
            status: StepStatus::Success,
            duration_ms: None,
            error_code: None,
            error_message: None,
            details: None,
        }
    }

    pub fn failure(error_code: impl Into<String>) -> Self {
        Self {
            status: StepStatus::Failure,
            duration_ms: None,
            error_code: Some(error_code.into()),
            error_message: None,
            details: None,
        }
    }

    pub fn skipped() -> Self {
        Self {
            status: StepStatus::Skipped,
            duration_ms: None,
            error_code: None,
            error_message: None,
            details: None,
        }
    }

    pub fn with_duration(self, duration_ms: i64) -> Self {
        Self {
            duration_ms: Some(duration_ms),
            ..self
        }
    }

    pub fn with_message(self, error_message: Option<String>) -> Self {
        Self {
            error_message,
            ..self
        }
    }

    pub fn with_details(self, details: serde_json::Value) -> Self {
        Self {
            details: Some(details),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_carries_no_error_code_and_no_details() {
        let outcome = StepOutcome::success();

        assert_eq!(outcome.status, StepStatus::Success);
        assert_eq!(outcome.error_code, None);
        assert_eq!(outcome.details, None);
    }

    #[test]
    fn failure_carries_the_given_error_code() {
        let outcome = StepOutcome::failure("boom");

        assert_eq!(outcome.status, StepStatus::Failure);
        assert_eq!(outcome.error_code, Some("boom".to_string()));
        assert_eq!(outcome.error_message, None);
        assert_eq!(outcome.details, None);
    }

    #[test]
    fn skipped_is_the_skipped_status() {
        let outcome = StepOutcome::skipped();

        assert_eq!(outcome.status, StepStatus::Skipped);
    }

    #[test]
    fn with_details_then_with_duration_both_survive() {
        let details = serde_json::json!({ "attempt": 1 });

        let outcome = StepOutcome::success()
            .with_details(details.clone())
            .with_duration(42);

        assert_eq!(outcome.details, Some(details));
        assert_eq!(outcome.duration_ms, Some(42));
    }

    #[test]
    fn with_duration_then_with_details_both_survive() {
        let details = serde_json::json!({ "attempt": 1 });

        let outcome = StepOutcome::success()
            .with_duration(42)
            .with_details(details.clone());

        assert_eq!(outcome.details, Some(details));
        assert_eq!(outcome.duration_ms, Some(42));
    }
}
