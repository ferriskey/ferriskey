use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use typeshare::typeshare;
use uuid::Uuid;

pub mod errors;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: i32,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: i32,
    #[typeshare(serialized_as = "string")]
    pub realm_id: Uuid,
    #[typeshare(serialized_as = "string", optional)]
    pub client_id: Option<Uuid>
}