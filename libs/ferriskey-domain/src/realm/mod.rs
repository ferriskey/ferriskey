pub mod ports;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
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
    /// Human-readable label shown in the login and admin UI.
    ///
    /// Unlike `name` (which is the URL slug and must stay URL-safe), this can
    /// contain whitespace and be changed freely without breaking realm URLs.
    /// Falls back to `name` when unset.
    pub display_name: Option<String>,
    pub settings: Option<RealmSetting>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoginAlias {
    Username,
    Email,
}

impl std::fmt::Display for LoginAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginAlias::Username => write!(f, "username"),
            LoginAlias::Email => write!(f, "email"),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoginAliasesError {
    #[error("login aliases list must not be empty")]
    Empty,
    #[error("login aliases list must not contain duplicates")]
    Duplicate,
    #[error("unknown login alias: {0}")]
    Unknown(String),
}

impl FromStr for LoginAlias {
    type Err = LoginAliasesError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "username" => Ok(LoginAlias::Username),
            "email" => Ok(LoginAlias::Email),
            other => Err(LoginAliasesError::Unknown(other.to_string())),
        }
    }
}

/// Ordered, non-empty, duplicate-free list of login identifiers a realm accepts.
/// Order encodes precedence: the first alias that resolves a user wins.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "Vec<LoginAlias>", into = "Vec<LoginAlias>")]
#[schema(value_type = Vec<LoginAlias>)]
pub struct LoginAliases(Vec<LoginAlias>);

impl LoginAliases {
    pub fn try_new(aliases: Vec<LoginAlias>) -> Result<Self, LoginAliasesError> {
        if aliases.is_empty() {
            return Err(LoginAliasesError::Empty);
        }
        let mut seen = Vec::with_capacity(aliases.len());
        for alias in &aliases {
            if seen.contains(alias) {
                return Err(LoginAliasesError::Duplicate);
            }
            seen.push(*alias);
        }
        Ok(Self(aliases))
    }

    pub fn as_slice(&self) -> &[LoginAlias] {
        &self.0
    }
}

impl Default for LoginAliases {
    fn default() -> Self {
        Self(vec![LoginAlias::Username])
    }
}

impl TryFrom<Vec<LoginAlias>> for LoginAliases {
    type Error = LoginAliasesError;
    fn try_from(value: Vec<LoginAlias>) -> Result<Self, Self::Error> {
        LoginAliases::try_new(value)
    }
}

impl From<LoginAliases> for Vec<LoginAlias> {
    fn from(value: LoginAliases) -> Self {
        value.0
    }
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
    pub passkey_enabled: bool,
    pub compass_enabled: bool,
    pub access_token_lifetime: i64,
    pub refresh_token_lifetime: i64,
    pub id_token_lifetime: i64,
    pub temporary_token_lifetime: i64,
    pub reset_password_template_id: Option<Uuid>,
    pub magic_link_template_id: Option<Uuid>,
    pub email_verification_template_id: Option<Uuid>,
    pub email_verification_enabled: bool,
    pub email_verification_ttl_hours: i64,
    pub login_aliases: LoginAliases,
    pub require_mfa: bool,
    pub updated_at: DateTime<Utc>,
    pub lockout_threshold: i32,
    pub lockout_duration_seconds: i32,
    pub seawatch_pii_mode: String,
    #[serde(skip_serializing)]
    pub seawatch_pseudo_key: Option<String>,
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
            passkey_enabled: false,
            compass_enabled: true,
            access_token_lifetime: 300,
            refresh_token_lifetime: 86400,
            id_token_lifetime: 300,
            temporary_token_lifetime: 300,
            reset_password_template_id: None,
            magic_link_template_id: None,
            email_verification_template_id: None,
            email_verification_enabled: false,
            email_verification_ttl_hours: 24,
            login_aliases: LoginAliases::default(),
            require_mfa: false,
            updated_at: now,
            lockout_threshold: 10,
            lockout_duration_seconds: 900,
            seawatch_pii_mode: "off".to_string(),
            seawatch_pseudo_key: None,
        }
    }
}

impl Realm {
    pub fn new(name: String) -> Self {
        let now = Utc::now();

        Self {
            id: RealmId::default(),
            name,
            display_name: None,
            settings: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_delete(&self) -> bool {
        self.name != "master"
    }
}

#[cfg(test)]
mod login_alias_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parses_known_aliases() {
        assert_eq!(
            LoginAlias::from_str("username").unwrap(),
            LoginAlias::Username
        );
        assert_eq!(LoginAlias::from_str("email").unwrap(), LoginAlias::Email);
    }

    #[test]
    fn rejects_unknown_alias() {
        assert!(LoginAlias::from_str("phone").is_err());
    }

    #[test]
    fn display_roundtrips() {
        assert_eq!(LoginAlias::Username.to_string(), "username");
        assert_eq!(LoginAlias::Email.to_string(), "email");
    }

    #[test]
    fn try_new_rejects_empty() {
        assert!(matches!(
            LoginAliases::try_new(vec![]),
            Err(LoginAliasesError::Empty)
        ));
    }

    #[test]
    fn try_new_rejects_duplicates() {
        let dup = vec![LoginAlias::Email, LoginAlias::Email];
        assert!(matches!(
            LoginAliases::try_new(dup),
            Err(LoginAliasesError::Duplicate)
        ));
    }

    #[test]
    fn try_new_preserves_order() {
        let aliases = LoginAliases::try_new(vec![LoginAlias::Email, LoginAlias::Username]).unwrap();
        assert_eq!(
            aliases.as_slice(),
            &[LoginAlias::Email, LoginAlias::Username]
        );
    }

    #[test]
    fn default_is_username_only() {
        assert_eq!(LoginAliases::default().as_slice(), &[LoginAlias::Username]);
    }

    #[test]
    fn serde_rejects_empty_array() {
        let json = "[]";
        assert!(serde_json::from_str::<LoginAliases>(json).is_err());
    }

    #[test]
    fn serde_roundtrips_valid_list() {
        let json = r#"["email","username"]"#;
        let parsed: LoginAliases = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed.as_slice(),
            &[LoginAlias::Email, LoginAlias::Username]
        );
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }
}
