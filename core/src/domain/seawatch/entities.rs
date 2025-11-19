use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_uuid_v7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SecurityEventType {
    #[serde(rename = "login_success")]
    LoginSuccess,

    #[serde(rename = "login_failure")]
    LoginFailure,

    #[serde(rename = "password_reset")]
    PasswordReset,

    #[serde(rename = "user_created")]
    UserCreated,

    #[serde(rename = "user_deleted")]
    UserDeleted,

    #[serde(rename = "role_assigned")]
    RoleAssigned,

    #[serde(rename = "role_unassigned")]
    RoleUnassigned,

    #[serde(rename = "role_created")]
    RoleCreated,

    #[serde(rename = "role_removed")]
    RoleRemoved,

    #[serde(rename = "client_created")]
    ClientCreated,

    #[serde(rename = "client_deleted")]
    ClientDeleted,

    #[serde(rename = "client_secret_rotated")]
    ClientSecretRotated,

    #[serde(rename = "realm_config_changed")]
    RealmConfigChanged,
}

impl Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::LoginSuccess => write!(f, "login_success"),
            SecurityEventType::LoginFailure => write!(f, "login_failure"),
            SecurityEventType::PasswordReset => write!(f, "password_reset"),
            SecurityEventType::UserCreated => write!(f, "user_created"),
            SecurityEventType::UserDeleted => write!(f, "user_deleted"),
            SecurityEventType::RoleAssigned => write!(f, "role_assigned"),
            SecurityEventType::RoleUnassigned => write!(f, "role_unassigned"),
            SecurityEventType::RoleCreated => write!(f, "role_created"),
            SecurityEventType::RoleRemoved => write!(f, "role_removed"),
            SecurityEventType::ClientCreated => write!(f, "client_created"),
            SecurityEventType::ClientDeleted => write!(f, "client_deleted"),
            SecurityEventType::ClientSecretRotated => write!(f, "client_secret_rotated"),
            SecurityEventType::RealmConfigChanged => write!(f, "realm_config_changed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ActorType {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "service_account")]
    ServiceAccount,

    #[serde(rename = "admin")]
    Admin,

    #[serde(rename = "system")]
    System,
}

impl Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorType::User => write!(f, "user"),
            ActorType::ServiceAccount => write!(f, "service_account"),
            ActorType::Admin => write!(f, "admin"),
            ActorType::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum EventStatus {
    #[serde(rename = "success")]
    Success,

    #[serde(rename = "failure")]
    Failure,
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventStatus::Success => write!(f, "success"),
            EventStatus::Failure => write!(f, "failure"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SecurityEventId(pub Uuid);

impl SecurityEventId {
    pub fn new() -> Self {
        SecurityEventId(generate_uuid_v7())
    }
}

impl From<Uuid> for SecurityEventId {
    fn from(value: Uuid) -> Self {
        SecurityEventId(value)
    }
}

impl From<SecurityEventId> for Uuid {
    fn from(value: SecurityEventId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SecurityEvent {
    pub id: SecurityEventId,
    pub realm_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_type: Option<ActorType>,
    pub event_type: SecurityEventType,
    pub status: EventStatus,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub resource: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub trace_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl SecurityEvent {
    pub fn new(
        realm_id: Uuid,
        event_type: SecurityEventType,
        status: EventStatus,
        actor_id: Uuid,
    ) -> Self {
        Self {
            id: SecurityEventId::new(),
            realm_id,
            actor_id: Some(actor_id),
            actor_type: None,
            event_type,
            status,
            target_type: None,
            target_id: None,
            resource: None,
            timestamp: Utc::now(),
            trace_id: None,
            ip_address: None,
            user_agent: None,
            details: None,
        }
    }

    pub fn with_actor(mut self, actor_id: Uuid, actor_type: ActorType) -> Self {
        self.actor_id = Some(actor_id);
        self.actor_type = Some(actor_type);
        self
    }

    pub fn with_target(
        mut self,
        target_type: String,
        target_id: Uuid,
        resource: Option<String>,
    ) -> Self {
        self.target_type = Some(target_type);
        self.target_id = Some(target_id);
        self.resource = resource;
        self
    }

    pub fn with_context(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
        trace_id: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self.trace_id = trace_id;
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
