use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::client::entities::Client;

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
