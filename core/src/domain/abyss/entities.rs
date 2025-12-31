use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_uuid_v7;
use crate::domain::realm::entities::RealmId;

/// Unique identifier for a Provider
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct ProviderId(Uuid);

impl ProviderId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
}

impl Default for ProviderId {
    fn default() -> Self {
        Self::new(generate_uuid_v7())
    }
}

impl From<Uuid> for ProviderId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ProviderId> for Uuid {
    fn from(id: ProviderId) -> Self {
        id.0
    }
}

impl PartialEq<Uuid> for ProviderId {
    fn eq(&self, other: &Uuid) -> bool {
        self.0.eq(other)
    }
}

/// Type of external identity provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// OAuth2 providers (Google, GitHub, Discord, etc.)
    OAuth2,
    /// OpenID Connect providers
    Oidc,
    /// SAML providers (future support)
    Saml,
    /// LDAP directories (future support)
    Ldap,
}

impl ProviderType {
    /// Returns the string representation of the provider type
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OAuth2 => "oauth2",
            Self::Oidc => "oidc",
            Self::Saml => "saml",
            Self::Ldap => "ldap",
        }
    }

    /// Returns true if this provider type supports OAuth2 flows
    pub fn supports_oauth2_flow(&self) -> bool {
        matches!(self, Self::OAuth2 | Self::Oidc)
    }
}

impl FromStr for ProviderType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "oauth2" => Ok(Self::OAuth2),
            "oidc" => Ok(Self::Oidc),
            "saml" => Ok(Self::Saml),
            "ldap" => Ok(Self::Ldap),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// External Identity Provider configuration
///
/// Represents an external identity provider (IdP) such as Google, GitHub, Discord,
/// or enterprise providers like SAML/LDAP. Providers are scoped to a realm and
/// enable federated authentication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Provider {
    /// Unique identifier for the provider
    pub id: ProviderId,
    /// Realm this provider belongs to
    pub realm_id: RealmId,
    /// Display name for the provider
    pub name: String,
    /// Type of identity provider
    pub provider_type: ProviderType,
    /// OAuth2/OIDC client ID
    pub client_id: String,
    /// OAuth2/OIDC client secret (encrypted in storage)
    pub client_secret: String,
    /// Authorization endpoint URL
    pub authorization_url: String,
    /// Token endpoint URL
    pub token_url: String,
    /// UserInfo endpoint URL (optional for some providers)
    pub userinfo_url: Option<String>,
    /// OAuth2 scopes to request
    pub scopes: Vec<String>,
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Provider-specific configuration (e.g., tenant ID for Azure AD)
    pub configuration: serde_json::Value,
    /// When the provider was created
    pub created_at: DateTime<Utc>,
    /// When the provider was last updated
    pub updated_at: DateTime<Utc>,
}

/// Configuration for creating a new Provider
pub struct ProviderConfig {
    pub realm_id: RealmId,
    pub name: String,
    pub provider_type: ProviderType,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub scopes: Vec<String>,
    pub configuration: serde_json::Value,
}

impl Provider {
    /// Creates a new Provider with the given configuration
    pub fn new(config: ProviderConfig) -> Self {
        let now = Utc::now();

        Self {
            id: ProviderId::default(),
            realm_id: config.realm_id,
            name: config.name,
            provider_type: config.provider_type,
            client_id: config.client_id,
            client_secret: config.client_secret,
            authorization_url: config.authorization_url,
            token_url: config.token_url,
            userinfo_url: config.userinfo_url,
            scopes: config.scopes,
            enabled: true,
            configuration: config.configuration,
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if this provider can be used for authentication
    pub fn is_usable(&self) -> bool {
        self.enabled && self.provider_type.supports_oauth2_flow()
    }

    /// Updates the provider's enabled status
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// Updates the provider's name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Updates the OAuth2 credentials
    pub fn update_credentials(&mut self, client_id: String, client_secret: String) {
        self.client_id = client_id;
        self.client_secret = client_secret;
        self.updated_at = Utc::now();
    }

    /// Updates the endpoint URLs
    pub fn update_urls(
        &mut self,
        authorization_url: String,
        token_url: String,
        userinfo_url: Option<String>,
    ) {
        self.authorization_url = authorization_url;
        self.token_url = token_url;
        self.userinfo_url = userinfo_url;
        self.updated_at = Utc::now();
    }

    /// Updates the OAuth2 scopes
    pub fn set_scopes(&mut self, scopes: Vec<String>) {
        self.scopes = scopes;
        self.updated_at = Utc::now();
    }

    /// Updates the provider-specific configuration
    pub fn set_configuration(&mut self, configuration: serde_json::Value) {
        self.configuration = configuration;
        self.updated_at = Utc::now();
    }
}

/// Unique identifier for a ProviderMapping
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct ProviderMappingId(Uuid);

impl ProviderMappingId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
}

impl Default for ProviderMappingId {
    fn default() -> Self {
        Self::new(generate_uuid_v7())
    }
}

impl From<Uuid> for ProviderMappingId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ProviderMappingId> for Uuid {
    fn from(id: ProviderMappingId) -> Self {
        id.0
    }
}

/// Mapping between external provider attributes and internal user fields
///
/// Defines how attributes from the external identity provider (e.g., "email", "sub", "name")
/// should be mapped to internal FerrisKey user fields (e.g., "email", "username").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProviderMapping {
    /// Unique identifier for the mapping
    pub id: ProviderMappingId,
    /// Provider this mapping belongs to
    pub provider_id: ProviderId,
    /// External field name from the provider (e.g., "email", "sub", "preferred_username")
    pub external_field: String,
    /// Internal FerrisKey field name (e.g., "email", "username", "first_name")
    pub internal_field: String,
    /// Whether this mapping is required for successful authentication
    pub is_required: bool,
}

/// Configuration for creating a new ProviderMapping
pub struct ProviderMappingConfig {
    pub provider_id: ProviderId,
    pub external_field: String,
    pub internal_field: String,
    pub is_required: bool,
}

impl ProviderMapping {
    /// Creates a new ProviderMapping with the given configuration
    pub fn new(config: ProviderMappingConfig) -> Self {
        Self {
            id: ProviderMappingId::default(),
            provider_id: config.provider_id,
            external_field: config.external_field,
            internal_field: config.internal_field,
            is_required: config.is_required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_as_str() {
        assert_eq!(ProviderType::OAuth2.as_str(), "oauth2");
        assert_eq!(ProviderType::Oidc.as_str(), "oidc");
        assert_eq!(ProviderType::Saml.as_str(), "saml");
        assert_eq!(ProviderType::Ldap.as_str(), "ldap");
    }

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(ProviderType::from_str("oauth2"), Some(ProviderType::OAuth2));
        assert_eq!(ProviderType::from_str("OIDC"), Some(ProviderType::Oidc));
        assert_eq!(ProviderType::from_str("invalid"), None);
    }

    #[test]
    fn test_provider_type_supports_oauth2_flow() {
        assert!(ProviderType::OAuth2.supports_oauth2_flow());
        assert!(ProviderType::Oidc.supports_oauth2_flow());
        assert!(!ProviderType::Saml.supports_oauth2_flow());
        assert!(!ProviderType::Ldap.supports_oauth2_flow());
    }

    #[test]
    fn test_provider_new() {
        let realm_id = RealmId::default();
        let provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client123".to_string(),
            client_secret: "secret456".to_string(),
            authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string()],
            configuration: serde_json::json!({}),
        });

        assert_eq!(provider.name, "Google");
        assert_eq!(provider.provider_type, ProviderType::OAuth2);
        assert!(provider.enabled);
        assert!(provider.is_usable());
    }

    #[test]
    fn test_provider_set_enabled() {
        let realm_id = RealmId::default();
        let mut provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Test".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: None,
            scopes: vec![],
            configuration: serde_json::json!({}),
        });

        assert!(provider.is_usable());
        provider.set_enabled(false);
        assert!(!provider.is_usable());
    }

    #[test]
    fn test_provider_mapping_new() {
        let provider_id = ProviderId::default();
        let mapping = ProviderMapping::new(ProviderMappingConfig {
            provider_id,
            external_field: "email".to_string(),
            internal_field: "email".to_string(),
            is_required: true,
        });

        assert_eq!(mapping.external_field, "email");
        assert_eq!(mapping.internal_field, "email");
        assert!(mapping.is_required);
    }

    #[test]
    fn test_provider_id_conversions() {
        let uuid = Uuid::new_v4();
        let provider_id = ProviderId::from(uuid);
        let back_to_uuid: Uuid = provider_id.into();
        assert_eq!(uuid, back_to_uuid);
    }
}
