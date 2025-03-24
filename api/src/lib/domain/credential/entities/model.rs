use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema, FromRow,
)]
pub struct Credential {
    pub id: Uuid,
    pub salt: Option<String>,
    pub credential_type: String,
    pub user_id: Uuid,
    pub user_label: Option<String>,
    pub secret_data: String,
    pub credential_data: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Credential {
    pub fn new(
        salt: String,
        credential_type: String,
        user_id: Uuid,
        user_label: String,
        secret_data: String,
        credential_data: String,
    ) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);

        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            salt: Some(salt),
            credential_type,
            user_id,
            user_label: Some(user_label),
            secret_data,
            credential_data,
            created_at: now,
            updated_at: now,
        }
    }
}
