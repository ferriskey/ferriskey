use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod permission;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
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
    #[error("Forbidden")]
    Forbidden,
}
