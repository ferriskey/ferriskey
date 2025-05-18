use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::client::entities::model::Client;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, FromRow, ToSchema,
)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: i64,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub client: Option<Client>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
