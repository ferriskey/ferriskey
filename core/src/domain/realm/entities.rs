use chrono::{DateTime, Utc};
pub use ferriskey_domain::realm::{Realm, RealmId, RealmSetting};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::abyss::identity_provider::entities::IdentityProviderPresentation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SmtpConfig {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub encryption: SmtpEncryption,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SmtpEncryption {
    Tls,
    StartTls,
    None,
}

impl SmtpEncryption {
    pub fn as_str(&self) -> &str {
        match self {
            SmtpEncryption::Tls => "tls",
            SmtpEncryption::StartTls => "starttls",
            SmtpEncryption::None => "none",
        }
    }
}

impl std::str::FromStr for SmtpEncryption {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "starttls" => SmtpEncryption::StartTls,
            "none" => SmtpEncryption::None,
            _ => SmtpEncryption::Tls,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RealmLoginSetting {
    pub user_registration_enabled: bool,
    pub forgot_password_enabled: bool,
    pub remember_me_enabled: bool,
    pub identity_providers: Vec<IdentityProviderPresentation>,
    pub magic_link_enabled: bool,
    pub magic_link_ttl: u32,
    pub passkey_enabled: bool,
    pub theme: crate::domain::portal_theme::entities::PortalThemeConfig,
}

impl From<RealmSetting> for RealmLoginSetting {
    fn from(value: RealmSetting) -> Self {
        Self {
            forgot_password_enabled: value.forgot_password_enabled,
            remember_me_enabled: value.remember_me_enabled,
            user_registration_enabled: value.user_registration_enabled,
            identity_providers: Vec::new(),
            magic_link_enabled: value.magic_link_enabled,
            magic_link_ttl: value.magic_link_ttl,
            passkey_enabled: value.passkey_enabled,
            theme: crate::domain::portal_theme::entities::PortalThemeConfig::default(),
        }
    }
}
