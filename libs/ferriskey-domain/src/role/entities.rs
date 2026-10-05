use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::client::entities::Client;
use crate::realm::RealmId;
use crate::realm::scope::RealmOwned;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub realm_id: RealmId,
    pub client_id: Option<Uuid>,
    pub client: Option<Client>,
    pub require_mfa: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RealmOwned for Role {
    fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoleFilter {
    pub search: Option<String>,
    pub name: Option<String>,
    pub qualified_name: Option<String>,
    pub description: Option<String>,
    pub require_mfa: Option<bool>,
    pub client_id: Option<Uuid>,
    pub scope: Option<RoleScope>,
    pub has_permissions: Option<bool>,
    pub ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleScope {
    Realm,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleSortField {
    Name,
    #[default]
    CreatedAt,
    UpdatedAt,
}
