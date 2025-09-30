use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SecurityEventType {
    #[serde(rename = "login_success")]
    LoginSuccess,

    #[serde(rename = "login_failure")]
    LoginFailure,

    #[serde(rename = "role_assigned")]
    RoleAssigned,

    #[serde(rename = "role_unassigned")]
    RoleUnassigned,

    #[serde(rename = "role_created")]
    RoleCreated,

    #[serde(rename = "role_removed")]
    RoleRemoved,

    #[serde(rename = "realm_config_changed")]
    RealmConfigChanged,

    #[serde(rename = "client_secret_rotated")]
    ClientSecretRotated,
}

impl Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::LoginSuccess => write!(f, "login_success"),
            SecurityEventType::LoginFailure => write!(f, "login_failure"),
            SecurityEventType::RoleAssigned => write!(f, "role_assigned"),
            SecurityEventType::RoleUnassigned => write!(f, "role_unassigned"),
            SecurityEventType::RoleCreated => write!(f, "role_created"),
            SecurityEventType::RoleRemoved => write!(f, "role_removed"),
            SecurityEventType::RealmConfigChanged => write!(f, "realm_config_changed"),
            SecurityEventType::ClientSecretRotated => write!(f, "client_secret_rotated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub event_type: SecurityEventType,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}