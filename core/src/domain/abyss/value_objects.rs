use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::abyss::entities::ProviderType;
use crate::domain::realm::entities::RealmId;

/// Input for creating a new identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProviderInput {
    /// Realm to create the provider in
    pub realm_id: RealmId,
    /// Display name for the provider
    pub name: String,
    /// Type of identity provider
    pub provider_type: ProviderType,
    /// OAuth2/OIDC client ID
    pub client_id: String,
    /// OAuth2/OIDC client secret
    pub client_secret: String,
    /// Authorization endpoint URL
    pub authorization_url: String,
    /// Token endpoint URL
    pub token_url: String,
    /// UserInfo endpoint URL (optional)
    pub userinfo_url: Option<String>,
    /// OAuth2 scopes to request
    pub scopes: Vec<String>,
    /// Provider-specific configuration
    pub configuration: serde_json::Value,
}

/// Input for updating an existing identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProviderInput {
    /// ID of the provider to update
    pub id: Uuid,
    /// New display name (optional)
    pub name: Option<String>,
    /// New OAuth2/OIDC client ID (optional)
    pub client_id: Option<String>,
    /// New OAuth2/OIDC client secret (optional)
    pub client_secret: Option<String>,
    /// New authorization endpoint URL (optional)
    pub authorization_url: Option<String>,
    /// New token endpoint URL (optional)
    pub token_url: Option<String>,
    /// New UserInfo endpoint URL (optional)
    pub userinfo_url: Option<String>,
    /// New OAuth2 scopes (optional)
    pub scopes: Option<Vec<String>>,
    /// New provider-specific configuration (optional)
    pub configuration: Option<serde_json::Value>,
}

/// Input for retrieving a provider by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProviderInput {
    /// ID of the provider to retrieve
    pub id: Uuid,
}

/// Input for deleting a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProviderInput {
    /// ID of the provider to delete
    pub id: Uuid,
}

/// Input for enabling or disabling a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleProviderInput {
    /// ID of the provider to toggle
    pub id: Uuid,
    /// New enabled status
    pub enabled: bool,
}

/// Input for creating a provider attribute mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMappingInput {
    /// ID of the provider this mapping belongs to
    pub provider_id: Uuid,
    /// External field name from the provider
    pub external_field: String,
    /// Internal FerrisKey field name
    pub internal_field: String,
    /// Whether this mapping is required
    pub is_required: bool,
}

/// Input for deleting a provider attribute mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMappingInput {
    /// ID of the mapping to delete
    pub id: Uuid,
}

/// Input for retrieving providers by realm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProvidersByRealmInput {
    /// ID of the realm to get providers for
    pub realm_id: RealmId,
}

/// Input for retrieving mappings by provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMappingsByProviderInput {
    /// ID of the provider to get mappings for
    pub provider_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_input() {
        let input = CreateProviderInput {
            realm_id: RealmId::default(),
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client123".to_string(),
            client_secret: "secret456".to_string(),
            authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string()],
            configuration: serde_json::json!({}),
        };

        assert_eq!(input.name, "Google");
        assert_eq!(input.provider_type, ProviderType::OAuth2);
    }

    #[test]
    fn test_update_provider_input_partial() {
        let input = UpdateProviderInput {
            id: Uuid::new_v4(),
            name: Some("Updated Name".to_string()),
            client_id: None,
            client_secret: None,
            authorization_url: None,
            token_url: None,
            userinfo_url: None,
            scopes: None,
            configuration: None,
        };

        assert!(input.name.is_some());
        assert!(input.client_id.is_none());
    }

    #[test]
    fn test_toggle_provider_input() {
        let input = ToggleProviderInput {
            id: Uuid::new_v4(),
            enabled: false,
        };

        assert!(!input.enabled);
    }

    #[test]
    fn test_create_mapping_input() {
        let input = CreateMappingInput {
            provider_id: Uuid::new_v4(),
            external_field: "email".to_string(),
            internal_field: "email".to_string(),
            is_required: true,
        };

        assert_eq!(input.external_field, "email");
        assert!(input.is_required);
    }
}
