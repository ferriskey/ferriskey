use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::client::entities::MaintenanceSessionStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleMaintenanceRequest {
    pub enabled: bool,
    pub reason: Option<String>,
    pub session_strategy: Option<MaintenanceSessionStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMaintenanceWhitelistUserRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMaintenanceWhitelistRoleRequest {
    pub role_id: Uuid,
}
