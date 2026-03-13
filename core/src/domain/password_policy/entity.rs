use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PasswordPolicy {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
    pub max_age_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
    pub max_age_days: Option<i32>,
}
