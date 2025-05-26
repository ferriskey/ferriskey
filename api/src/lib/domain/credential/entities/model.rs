use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub fn new(
        id: Uuid,
        salt: String,
        credential_type: String,
        user_id: Uuid,
        user_label: String,
        secret_data: String,
        credential_data: CredentialData,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            salt: Some(salt),
            credential_type,
            user_id,
            user_label: Some(user_label),
            secret_data,
            credential_data,
            created_at,
            updated_at,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow, PartialOrd, Ord,
)]
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
