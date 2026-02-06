use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::identity_provider::IdentityProviderId;
use ferriskey_domain::realm::RealmId;

/// Broker authentication session - tracks OAuth state during SSO flow
///
/// This session is created when a user initiates SSO login and stores
/// the necessary state to complete the OAuth flow when the IdP redirects back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAuthSession {
    /// Unique identifier for this session
    pub id: Uuid,

    /// Realm this session belongs to
    pub realm_id: RealmId,

    /// Identity provider being used for authentication
    pub identity_provider_id: IdentityProviderId,

    /// Client application that initiated the login
    pub client_id: Uuid,

    /// Original redirect URI to return to after authentication
    pub redirect_uri: String,

    /// OAuth response type (typically "code")
    pub response_type: String,

    /// Requested scopes
    pub scope: String,

    /// Original OAuth state from the client
    pub state: Option<String>,

    /// OIDC nonce for replay protection
    pub nonce: Option<String>,

    /// Random state sent to IdP (CSRF protection)
    pub broker_state: String,

    /// PKCE code verifier (if PKCE is enabled)
    pub code_verifier: Option<String>,

    /// Linked FerrisKey auth session ID (if initiated from existing auth flow)
    pub auth_session_id: Option<Uuid>,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
}

/// Parameters for creating a new BrokerAuthSession
pub struct BrokerAuthSessionParams {
    pub realm_id: RealmId,
    pub identity_provider_id: IdentityProviderId,
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

impl BrokerAuthSession {
    /// Creates a new BrokerAuthSession with the given parameters
    pub fn new(params: BrokerAuthSessionParams) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: params.realm_id,
            identity_provider_id: params.identity_provider_id,
            client_id: params.client_id,
            redirect_uri: params.redirect_uri,
            response_type: params.response_type,
            scope: params.scope,
            state: params.state,
            nonce: params.nonce,
            broker_state: params.broker_state,
            code_verifier: params.code_verifier,
            auth_session_id: params.auth_session_id,
            created_at: now,
            expires_at: now + Duration::minutes(10),
        }
    }

    /// Checks if this session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Links a FerrisKey user to an external identity provider identity
///
/// This mapping allows users to log in via SSO and be recognized
/// as existing FerrisKey users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderLink {
    /// Unique identifier for this link
    pub id: Uuid,

    /// FerrisKey user ID
    pub user_id: Uuid,

    /// Identity provider this link is for
    pub identity_provider_id: IdentityProviderId,

    /// User's unique identifier at the IdP (subject claim)
    pub identity_provider_user_id: String,

    /// User's username at the IdP
    pub identity_provider_username: String,

    /// Stored access token (if store_token is enabled)
    pub token: Option<String>,

    /// Link creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Parameters for creating a new IdentityProviderLink
pub struct IdentityProviderLinkParams {
    pub user_id: Uuid,
    pub identity_provider_id: IdentityProviderId,
    pub identity_provider_user_id: String,
    pub identity_provider_username: String,
    pub token: Option<String>,
}

impl IdentityProviderLink {
    /// Creates a new IdentityProviderLink with the given parameters
    pub fn new(params: IdentityProviderLinkParams) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            user_id: params.user_id,
            identity_provider_id: params.identity_provider_id,
            identity_provider_user_id: params.identity_provider_user_id,
            identity_provider_username: params.identity_provider_username,
            token: params.token,
            created_at: now,
            updated_at: now,
        }
    }

    /// Updates the stored token
    pub fn update_token(&mut self, token: Option<String>) {
        self.token = token;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broker_auth_session_new() {
        let params = BrokerAuthSessionParams {
            realm_id: RealmId::from(Uuid::new_v4()),
            identity_provider_id: IdentityProviderId::from(Uuid::new_v4()),
            client_id: Uuid::new_v4(),
            redirect_uri: "https://app.example.com/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid email profile".to_string(),
            state: Some("client-state".to_string()),
            nonce: Some("nonce123".to_string()),
            broker_state: "random-broker-state".to_string(),
            code_verifier: Some("pkce-verifier".to_string()),
            auth_session_id: None,
        };

        let session = BrokerAuthSession::new(params);

        assert_eq!(session.response_type, "code");
        assert!(!session.is_expired());
    }

    #[test]
    fn test_identity_provider_link_new() {
        let params = IdentityProviderLinkParams {
            user_id: Uuid::new_v4(),
            identity_provider_id: IdentityProviderId::from(Uuid::new_v4()),
            identity_provider_user_id: "google-user-123".to_string(),
            identity_provider_username: "user@gmail.com".to_string(),
            token: Some("access-token".to_string()),
        };

        let link = IdentityProviderLink::new(params);

        assert_eq!(link.identity_provider_user_id, "google-user-123");
        assert!(link.token.is_some());
    }

    #[test]
    fn test_identity_provider_link_update_token() {
        let params = IdentityProviderLinkParams {
            user_id: Uuid::new_v4(),
            identity_provider_id: IdentityProviderId::from(Uuid::new_v4()),
            identity_provider_user_id: "google-user-123".to_string(),
            identity_provider_username: "user@gmail.com".to_string(),
            token: None,
        };

        let mut link = IdentityProviderLink::new(params);
        assert!(link.token.is_none());

        link.update_token(Some("new-token".to_string()));
        assert_eq!(link.token, Some("new-token".to_string()));
    }
}
