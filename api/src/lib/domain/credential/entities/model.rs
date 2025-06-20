use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::credential::entities::credential_config::CredentialConfig;

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
    pub credential_data: CredentialData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Credential {
    pub fn new(config: CredentialConfig) -> Self {
        Self {
            id: config.id,
            salt: config.salt,
            credential_type: config.credential_type,
            user_id: config.user_id,
            user_label: config.user_label,
            secret_data: config.secret_data,
            credential_data: config.credential_data,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow, PartialOrd, Ord,
)]
#[typeshare]
pub struct CredentialData {
    pub hash_iterations: u32,
    pub algorithm: String,
}

impl CredentialData {
    pub fn new(hash_iterations: u32, algorithm: String) -> Self {
        Self {
            hash_iterations,
            algorithm,
        }
    }
}
