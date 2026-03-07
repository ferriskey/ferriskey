use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEvent {
    pub event_type: String,
    pub realm_id: String,
    pub resource_id: String,
    pub data: Option<serde_json::Value>,
}
