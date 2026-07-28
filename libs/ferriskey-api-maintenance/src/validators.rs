use ferriskey_core::domain::client::entities::MaintenanceSessionStrategy;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ToggleMaintenanceValidator {
    pub enabled: bool,

    #[serde(default)]
    pub reason: Option<String>,

    #[serde(default)]
    pub session_strategy: Option<MaintenanceSessionStrategy>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddWhitelistEntryValidator {
    #[serde(default)]
    pub user_id: Option<Uuid>,

    #[serde(default)]
    pub role_id: Option<Uuid>,
}
