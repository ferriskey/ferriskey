use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::{NoContext, Timestamp, Uuid};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct RedirectUri {
    pub id: Uuid,
    pub client_id: Uuid,
    pub value: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRedirectUriSchema {
    pub value: String,
    pub enabled: bool,
}

impl RedirectUri {
    pub fn new(client_id: Uuid, value: String, enabled: bool) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            client_id,
            value,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }
}
