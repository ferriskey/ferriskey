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
    /// Minimum Shannon entropy in bits (charset-pool model). CNIL 2022-100 default: 80.
    pub min_entropy_bits: i32,
    /// Reject passwords found in the embedded common-password list or matching username/email.
    pub forbid_common: bool,
    /// Interface flag — reject passwords reported as breached (requires external check; no
    /// network call is made when the provider is not wired up).
    pub check_breached: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PasswordPolicy {
    /// CNIL délibération 2022-100 compliant defaults (>= 80 bits entropy, all classes required,
    /// min 12 characters).
    pub fn default(realm_id: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            realm_id,
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            min_entropy_bits: 80,
            forbid_common: true,
            check_breached: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
    pub max_age_days: Option<i32>,
    pub min_entropy_bits: Option<i32>,
    pub forbid_common: Option<bool>,
    pub check_breached: Option<bool>,
}
