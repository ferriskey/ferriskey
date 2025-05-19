use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::client::entities::model::Client;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, FromRow, ToSchema,
)]
#[typeshare]
pub struct Role {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[typeshare(serialized_as = "number")]
    pub permissions: i64,
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
