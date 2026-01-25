use crate::domain::{
    common::entities::app_errors::CoreError, identity_provider::IdentityProviderConfig,
};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

/// OAuth2/OIDC provider configuration extracted from the identity provider's config JSONB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// OAuth client ID
    pub client_id: String,

    /// OAuth client secret
    pub client_secret: String,

    /// Authorization endpoint URL
    pub authorization_url: String,

    /// Token endpoint URL
    pub token_url: String,

    /// Userinfo endpoint URL (optional, can extract from ID token)
    pub userinfo_url: Option<String>,

    /// JWKS endpoint URL for ID token validation
    pub jwks_url: Option<String>,

    /// OAuth scopes to request
    #[serde(deserialize_with = "deserialize_scopes")]
    pub scopes: Vec<String>,

    /// Default scopes if none specified
    pub default_scopes: Option<String>,

    /// Whether to use PKCE
    pub use_pkce: Option<bool>,

    /// Expected issuer for ID token validation
    pub issuer: Option<String>,
}

fn deserialize_scopes<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) => Ok(s.split_whitespace().map(String::from).collect()),
        StringOrVec::Vec(v) => Ok(v),
    }
}

impl TryFrom<serde_json::Value> for OAuthProviderConfig {
    type Error = CoreError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        info!("value: {}", value);
        serde_json::from_value(value).map_err(|e| {
            CoreError::InvalidProviderConfiguration(format!(
                "Failed to parse OAuth provider config: {}",
                e
            ))
        })
    }
}

impl TryFrom<IdentityProviderConfig> for OAuthProviderConfig {
    type Error = CoreError;

    fn try_from(config: IdentityProviderConfig) -> Result<Self, Self::Error> {
        let mut extra = match config.extra {
            serde_json::Value::Object(map) => map,
            serde_json::Value::Null => serde_json::Map::new(),
            _ => {
                return Err(CoreError::InvalidProviderConfiguration(
                    "Provider configuration must be a JSON object".to_string(),
                ));
            }
        };

        info!("debug 1");

        let client_id = config.client_id.ok_or_else(|| {
            CoreError::InvalidProviderConfiguration("Missing client_id".to_string())
        })?;
        let client_secret = config.client_secret.ok_or_else(|| {
            CoreError::InvalidProviderConfiguration("Missing client_secret".to_string())
        })?;

        extra.insert(
            "client_id".to_string(),
            serde_json::Value::String(client_id),
        );
        extra.insert(
            "client_secret".to_string(),
            serde_json::Value::String(client_secret.expose().clone()),
        );

        OAuthProviderConfig::try_from(serde_json::Value::Object(extra))
    }
}

/// Input for initiating broker login
#[derive(Debug, Clone)]
pub struct BrokerLoginInput {
    /// Realm name
    pub realm_name: String,

    /// Identity provider alias
    pub alias: String,

    /// Client ID from the original OAuth request
    pub client_id: String,

    /// Redirect URI to return to after authentication
    pub redirect_uri: String,

    /// OAuth response type (typically "code")
    pub response_type: String,

    /// Requested scopes
    pub scope: Option<String>,

    /// Client's state parameter
    pub state: Option<String>,

    /// OIDC nonce parameter
    pub nonce: Option<String>,

    /// Existing auth session ID (if initiated from login page)
    pub auth_session_id: Option<Uuid>,

    /// Base URL of the API server (e.g., "https://auth.example.com")
    pub base_url: String,
}

/// Output from broker login initiation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrokerLoginOutput {
    /// URL to redirect the user to for IdP authentication
    pub authorization_url: String,

    /// Broker session ID for tracking
    pub broker_session_id: Uuid,
}

/// Input for broker callback handling
#[derive(Debug, Clone)]
pub struct BrokerCallbackInput {
    /// Realm name
    pub realm_name: String,

    /// Identity provider alias
    pub alias: String,

    /// Authorization code from IdP
    pub code: Option<String>,

    /// State parameter from IdP (for CSRF validation)
    pub state: String,

    /// Error code from IdP (if authentication failed)
    pub error: Option<String>,

    /// Error description from IdP
    pub error_description: Option<String>,

    /// Base URL of the API server (e.g., "https://auth.example.com")
    pub base_url: String,
}

/// Output from broker callback handling
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrokerCallbackOutput {
    /// URL to redirect the user to (client's redirect_uri with code)
    pub redirect_url: String,

    /// Authorization code for the client
    pub authorization_code: String,

    /// FerrisKey user ID
    pub user_id: Uuid,

    /// Whether this is a newly created user
    pub is_new_user: bool,
}

/// User info extracted from IdP tokens
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrokeredUserInfo {
    /// IdP's unique user ID (subject claim)
    pub subject: String,

    /// User's email address
    pub email: Option<String>,

    /// Whether the email is verified
    pub email_verified: Option<bool>,

    /// Full name
    pub name: Option<String>,

    /// First/given name
    pub given_name: Option<String>,

    /// Last/family name
    pub family_name: Option<String>,

    /// Preferred username
    pub preferred_username: Option<String>,

    /// Profile picture URL
    pub picture: Option<String>,
}

impl BrokeredUserInfo {
    /// Get the best username to use for account creation
    pub fn get_username(&self, idp_alias: &str) -> String {
        self.preferred_username
            .clone()
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| format!("{}_{}", idp_alias, self.subject))
    }
}

/// OAuth token response from IdP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    /// Access token
    pub access_token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Token expiration in seconds
    pub expires_in: Option<u64>,

    /// Refresh token (optional)
    pub refresh_token: Option<String>,

    /// ID token (for OIDC)
    pub id_token: Option<String>,

    /// Scopes granted
    pub scope: Option<String>,
}

/// Request to create an identity provider link
#[derive(Debug, Clone)]
pub struct CreateIdentityProviderLinkRequest {
    pub user_id: Uuid,
    pub identity_provider_id: Uuid,
    pub identity_provider_user_id: String,
    pub identity_provider_username: String,
    pub token: Option<String>,
}

/// Request to create a broker auth session
#[derive(Debug, Clone)]
pub struct CreateBrokerAuthSessionRequest {
    pub realm_id: Uuid,
    pub identity_provider_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub broker_state: String,
    pub code_verifier: Option<String>,
    pub auth_session_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_oauth_provider_config_from_json() {
        let json = json!({
            "client_id": "my-client-id",
            "client_secret": "my-secret",
            "authorization_url": "https://accounts.google.com/o/oauth2/v2/auth",
            "token_url": "https://oauth2.googleapis.com/token",
            "userinfo_url": "https://openidconnect.googleapis.com/v1/userinfo",
            "scopes": ["openid", "email", "profile"],
            "use_pkce": true
        });

        let config: OAuthProviderConfig = json.try_into().unwrap();

        assert_eq!(config.client_id, "my-client-id");
        assert_eq!(
            config.authorization_url,
            "https://accounts.google.com/o/oauth2/v2/auth"
        );
        assert_eq!(config.scopes, vec!["openid", "email", "profile"]);
        assert_eq!(config.use_pkce, Some(true));
    }

    #[test]
    fn test_brokered_user_info_get_username() {
        let mut info = BrokeredUserInfo {
            subject: "12345".to_string(),
            ..Default::default()
        };

        // Falls back to alias_subject format
        assert_eq!(info.get_username("google"), "google_12345");

        // Prefers email if available
        info.email = Some("user@example.com".to_string());
        assert_eq!(info.get_username("google"), "user@example.com");

        // Prefers preferred_username if available
        info.preferred_username = Some("myuser".to_string());
        assert_eq!(info.get_username("google"), "myuser");
    }
}
