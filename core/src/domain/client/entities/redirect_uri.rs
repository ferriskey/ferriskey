use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
