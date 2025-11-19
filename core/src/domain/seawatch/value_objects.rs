use super::entities::SecurityEventType;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SecurityEventFilter {
    pub user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub event_types: Option<Vec<SecurityEventType>>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
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

impl Default for SecurityEventFilter {
    fn default() -> Self {
        Self {
            user_id: None,
            actor_id: None,
            client_id: None,
            event_types: None,
            from_timestamp: None,
            to_timestamp: None,
            ip_address: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}
