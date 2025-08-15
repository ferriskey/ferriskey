use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct RedirectUri {
    pub id: Uuid,
    pub client_id: Uuid,
    pub value: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Error)]
pub enum RedirectUriError {
    #[error("Redirect URI not found")]
    NotFound,
    #[error("Redirect URI already exists")]
    AlreadyExists,
    #[error("Invalid redirect URI")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Database error")]
    DatabaseError,
    #[error("Redirect URI not enabled")]
    NotEnabled,
    #[error("Redirect URI not valid")]
    NotValid,
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
