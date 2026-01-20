use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{client::Client, realm::RealmId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub realm_id: RealmId,
    pub client_id: Option<Uuid>,
    pub client: Option<Client>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
