use crate::domain::utils::generate_timestamp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealmSetting {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub default_signing_algorithm: String,
    pub updated_at: DateTime<Utc>,
}

impl RealmSetting {
    pub fn new(realm_id: Uuid, default_signing_algorithm: String) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            default_signing_algorithm,
            updated_at: now,
        }
    }
}
