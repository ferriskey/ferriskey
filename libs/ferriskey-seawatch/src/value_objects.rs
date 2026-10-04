use super::entities::{EventStatus, SecurityEventType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecurityEventFilter {
    pub client_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub event_types: Option<Vec<SecurityEventType>>,
    pub status: Option<EventStatus>,
    pub target_type: Option<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventSortField {
    EventType,
    Status,
    Timestamp,
    #[default]
    CreatedAt,
}

#[derive(Debug, Clone)]
pub struct EventExportRequest {
    pub realm_id: Uuid,
    pub filter: SecurityEventFilter,
    pub format: ExportFormat,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Xlsx,
}
