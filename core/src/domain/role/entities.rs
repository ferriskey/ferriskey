use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::client::entities::Client;

pub mod permission;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
#[typeshare]
pub struct Role {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    #[typeshare(serialized_as = "string")]
    pub realm_id: Uuid,
    #[typeshare(serialized_as = "string")]
    pub client_id: Option<Uuid>,
    pub client: Option<Client>,
    #[typeshare(serialized_as = "Date")]
    pub created_at: DateTime<Utc>,
    #[typeshare(serialized_as = "Date")]
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
}
