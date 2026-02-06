use std::fmt::Display;

use chrono::{DateTime, Utc};
use maskass::Masked;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use ferriskey_domain::realm::RealmId;

/// Unique identifier for an Identity Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
pub struct IdentityProviderId(Uuid);

impl IdentityProviderId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for IdentityProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for IdentityProviderId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<IdentityProviderId> for Uuid {
    fn from(id: IdentityProviderId) -> Self {
        id.0
    }
}

/// Identity Provider entity
/// Represents an external authentication provider (Google, GitHub, OIDC, SAML, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct IdentityProvider {
    /// Unique identifier
    pub id: IdentityProviderId,

    /// Realm this provider belongs to
    pub realm_id: RealmId,

    /// Unique alias within the realm (e.g., "google", "github")
    pub alias: String,

    /// Provider type identifier (e.g., "google", "github", "oidc", "saml")
    pub provider_id: String,

    /// Whether this provider is enabled
    pub enabled: bool,

    /// Display name shown to users
    pub display_name: Option<String>,

    /// Authentication flow to use on first broker login
    pub first_broker_login_flow_alias: Option<String>,

    /// Authentication flow to use after broker login
    pub post_broker_login_flow_alias: Option<String>,

    /// Whether to store the external token
    pub store_token: bool,

    /// Whether to add read token role on account creation
    pub add_read_token_role_on_create: bool,

    /// Whether to trust the email from this provider
    pub trust_email: bool,

    /// Whether this provider should only link to existing accounts
    pub link_only: bool,

    /// Provider-specific configuration (OAuth client_id, client_secret, etc.)
    /// Stored as structured config
    pub config: IdentityProviderConfig,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct IdentityProviderConfig {
    /// OAuth client ID
    pub client_id: Option<String>,

    /// OAuth client secret (always masked on output)
    pub client_secret: Option<Masked<String>>,

    /// Catch-all for provider-specific configuration
    #[serde(flatten)]
    pub extra: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, ToSchema)]
pub struct IdentityProviderPresentation {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub icon: String,
    pub login_url: String,
}

impl IdentityProviderPresentation {
    pub fn new(provider: IdentityProvider, realm_name: &str) -> Self {
        Self {
            display_name: provider.display_name.unwrap_or_default(),
            icon: provider.provider_id.clone(),
            id: provider.alias.clone(),
            kind: provider.provider_id,
            login_url: format!("/realms/{}/broker/{}/login", realm_name, provider.alias),
        }
    }
}

/// Configuration for creating a new IdentityProvider
pub struct IdentityProviderCreationConfig {
    pub realm_id: RealmId,
    pub alias: String,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: IdentityProviderConfig,
}

impl IdentityProvider {
    /// Creates a new IdentityProvider with the given configuration
    pub fn new(config: IdentityProviderCreationConfig) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: IdentityProviderId::new(Uuid::new_v7(timestamp)),
            realm_id: config.realm_id,
            alias: config.alias,
            provider_id: config.provider_id,
            enabled: config.enabled,
            display_name: config.display_name,
            first_broker_login_flow_alias: config.first_broker_login_flow_alias,
            post_broker_login_flow_alias: config.post_broker_login_flow_alias,
            store_token: config.store_token,
            add_read_token_role_on_create: config.add_read_token_role_on_create,
            trust_email: config.trust_email,
            link_only: config.link_only,
            config: config.config,
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if this provider is usable (enabled)
    pub fn is_usable(&self) -> bool {
        self.enabled
    }

    /// Updates the enabled status
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// Updates the configuration
    pub fn update_config(&mut self, config: IdentityProviderConfig) {
        self.config = config;
        self.updated_at = Utc::now();
    }
}

/// Input for creating an identity provider
pub struct CreateIdentityProviderInput {
    pub realm_name: String,
    pub alias: String,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: JsonValue,
}

/// Input for updating an identity provider
pub struct UpdateIdentityProviderInput {
    pub realm_name: String,
    pub alias: String,
    pub enabled: Option<bool>,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: Option<bool>,
    pub add_read_token_role_on_create: Option<bool>,
    pub trust_email: Option<bool>,
    pub link_only: Option<bool>,
    pub config: Option<JsonValue>,
}

/// Input for getting an identity provider
pub struct GetIdentityProviderInput {
    pub realm_name: String,
    pub alias: String,
}

/// Input for listing identity providers
pub struct ListIdentityProvidersInput {
    pub realm_name: String,
}

/// Input for deleting an identity provider
pub struct DeleteIdentityProviderInput {
    pub realm_name: String,
    pub alias: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_identity_provider_new() {
        let config = IdentityProviderCreationConfig {
            realm_id: RealmId::from(Uuid::new_v4()),
            alias: "google".to_string(),
            provider_id: "google".to_string(),
            enabled: true,
            display_name: Some("Google".to_string()),
            first_broker_login_flow_alias: None,
            post_broker_login_flow_alias: None,
            store_token: false,
            add_read_token_role_on_create: false,
            trust_email: true,
            link_only: false,
            config: IdentityProviderConfig {
                client_id: Some("test".to_string()),
                client_secret: Some(Masked::new("secret".to_string())),
                extra: json!({}),
            },
        };

        let provider = IdentityProvider::new(config);

        assert_eq!(provider.alias, "google");
        assert_eq!(provider.provider_id, "google");
        assert!(provider.enabled);
        assert_eq!(provider.display_name, Some("Google".to_string()));
        assert!(provider.trust_email);
    }

    #[test]
    fn test_identity_provider_is_usable() {
        let config = IdentityProviderCreationConfig {
            realm_id: RealmId::from(Uuid::new_v4()),
            alias: "github".to_string(),
            provider_id: "github".to_string(),
            enabled: true,
            display_name: None,
            first_broker_login_flow_alias: None,
            post_broker_login_flow_alias: None,
            store_token: false,
            add_read_token_role_on_create: false,
            trust_email: false,
            link_only: false,
            config: IdentityProviderConfig {
                client_id: None,
                client_secret: None,
                extra: json!({}),
            },
        };

        let provider = IdentityProvider::new(config);
        assert!(provider.is_usable());
    }

    #[test]
    fn test_identity_provider_set_enabled() {
        let config = IdentityProviderCreationConfig {
            realm_id: RealmId::from(Uuid::new_v4()),
            alias: "oidc".to_string(),
            provider_id: "oidc".to_string(),
            enabled: true,
            display_name: None,
            first_broker_login_flow_alias: None,
            post_broker_login_flow_alias: None,
            store_token: false,
            add_read_token_role_on_create: false,
            trust_email: false,
            link_only: false,
            config: IdentityProviderConfig {
                client_id: None,
                client_secret: None,
                extra: json!({}),
            },
        };

        let mut provider = IdentityProvider::new(config);
        assert!(provider.enabled);

        provider.set_enabled(false);
        assert!(!provider.enabled);
        assert!(!provider.is_usable());
    }

    #[test]
    fn test_identity_provider_id_conversion() {
        let uuid = Uuid::new_v4();
        let id = IdentityProviderId::from(uuid);
        let back: Uuid = id.into();
        assert_eq!(uuid, back);
    }
}
