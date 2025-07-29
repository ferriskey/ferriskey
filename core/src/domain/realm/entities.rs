use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Realm {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Realm {
    pub fn new(name: String) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_delete(&self) -> bool {
        self.name != "master"
    }
}
