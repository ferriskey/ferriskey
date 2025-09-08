use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{client::entities::Client, webhook::entities::errors::WebhookError};

pub mod permission;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub client: Option<Client>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Error)]
pub enum RoleError {
    #[error("Role not found")]
    NotFound,

    #[error("Role already exists")]
    AlreadyExists,

    #[error("Invalid role")]
    Invalid,

    #[error("Internal server error")]
    InternalServerError,

    #[error("{0}")]
    Forbidden(String),

    #[error("Failed to notify webhook : {0}")]
    FailedWebhookNotification(WebhookError),
}

pub struct UpdateRoleInput {
    pub realm_name: String,
    pub role_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct GetUserRolesInput {
    pub realm_name: String,
    pub user_id: Uuid,
}
