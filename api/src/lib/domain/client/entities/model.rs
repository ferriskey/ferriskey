use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};
use validator::Validate;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, FromRow, ToSchema,
)]
pub struct Client {
    pub id: Uuid,
    pub enabled: bool,
    pub client_id: String,
    pub secret: Option<String>,
    pub realm_id: Uuid,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub client_type: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateClientSchema {
    pub name: String,
    pub client_id: String,
    pub enabled: bool,
    pub protocol: String,
}

impl Client {
    pub fn new(
        realm_id: Uuid,
        name: String,
        client_id: String,
        secret: Option<String>,
        enabled: bool,
        protocol: String,
        public_client: bool,
        service_account_enabled: bool,
        client_type: String,
    ) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);

        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
        Self {
            id: Uuid::new_v7(timestamp),
            enabled,
            client_id,
            secret,
            realm_id,
            protocol,
            public_client,
            service_account_enabled,
            client_type,
            name,
            created_at: now,
            updated_at: now,
        }
    }
}
