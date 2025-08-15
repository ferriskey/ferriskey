use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct Realm {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct RealmSetting {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub default_signing_algorithm: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Error)]
pub enum RealmError {
    #[error("Realm not found")]
    NotFound,
    #[error("Realm already exists")]
    AlreadyExists,
    #[error("Invalid realm")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Cannot delete master realm")]
    CannotDeleteMaster,
    #[error("Forbidden")]
    Forbidden,
}

impl RealmSetting {
    pub fn new(realm_id: Uuid, default_signing_algorithm: Option<String>) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            default_signing_algorithm,
            updated_at: now,
        }
    }
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
