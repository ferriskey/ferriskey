use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FlowFilter {
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: Option<String>,
    pub status: Option<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for FlowFilter {
    fn default() -> Self {
        Self {
            client_id: None,
            user_id: None,
            grant_type: None,
            status: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

pub struct FetchFlowsInput {
    pub realm_name: String,
    pub filter: FlowFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlowStats {
    pub total: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub pending_count: i64,
    pub avg_duration_ms: Option<f64>,
}
