use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{generate_timestamp, generate_uuid_v7};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct RealmId(Uuid);

impl RealmId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
}

impl Default for RealmId {
    fn default() -> Self {
        Self::new(generate_uuid_v7())
    }
}

impl From<Uuid> for RealmId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<RealmId> for Uuid {
    fn from(id: RealmId) -> Self {
        id.0
    }
}

impl PartialEq<Uuid> for RealmId {
    fn eq(&self, other: &Uuid) -> bool {
        self.0.eq(other)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct Realm {
    pub id: RealmId,
    pub name: String,
    pub settings: Option<RealmSetting>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct RealmSetting {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub default_signing_algorithm: Option<String>,
    pub user_registration_enabled: bool,
    pub forgot_password_enabled: bool,
    pub remember_me_enabled: bool,
    pub magic_link_enabled: bool,
    pub magic_link_ttl: u32,
    pub compass_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl RealmSetting {
    pub fn new(realm_id: RealmId, default_signing_algorithm: Option<String>) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            default_signing_algorithm,
            forgot_password_enabled: false,
            remember_me_enabled: false,
            user_registration_enabled: false,
            magic_link_enabled: false,
            magic_link_ttl: 15,
            compass_enabled: true,
            updated_at: now,
        }
    }
}

impl Realm {
    pub fn new(name: String) -> Self {
        let now = Utc::now();

        Self {
            id: RealmId::default(),
            name,
            settings: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_delete(&self) -> bool {
        self.name != "master"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct PasswordPolicy {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
    pub max_age_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PasswordPolicy {
    pub fn default(realm_id: RealmId) -> Self {
        let (now, _) = generate_timestamp();
        Self {
            id: generate_uuid_v7(),
            realm_id,
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdatePasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
    /// Tri-state: None = omitted, Some(None) = clear value, Some(Some(v)) = set value
    pub max_age_days: Option<Option<i32>>,
}
