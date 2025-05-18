use crate::domain::utils::generate_timestamp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, FromRow, ToSchema,
)]
#[typeshare]
pub struct Realm {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    pub name: String,
    #[typeshare(serialized_as = "Date")]
    pub created_at: DateTime<Utc>,
    #[typeshare(serialized_as = "Date")]
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
