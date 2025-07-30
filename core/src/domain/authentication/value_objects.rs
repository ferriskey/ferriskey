use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    authentication::entities::GrantType,
    client::entities::Client,
    user::entities::{RequiredAction, User},
};

pub struct AuthenticateRequest {
    pub realm_name: String,
    pub grant_type: GrantType,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateAuthSessionRequest {
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
}

pub struct GrantTypeParams {
    pub realm_id: Uuid,
    pub base_url: String,
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub code: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub user_id: Uuid,
    pub token: Option<String>,
    pub credentials: Vec<String>,
}

impl CreateAuthSessionRequest {
    pub fn new(realm_id: Uuid, client_id: Uuid, redirect_uri: String) -> Self {
        Self {
            realm_id,
            client_id,
            redirect_uri,
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: None,
            nonce: None,
            user_id: None,
        }
    }

    pub fn with_oauth_params(
        mut self,
        response_type: String,
        scope: String,
        state: Option<String>,
        nonce: Option<String>,
    ) -> Self {
        self.response_type = response_type;
        self.scope = scope;
        self.state = state;
        self.nonce = nonce;
        self
    }

    pub fn with_auth_info(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }
}

#[derive(Debug, Clone)]
pub enum Identity {
    User(User),
    Client(Client),
}

impl Identity {
    /// Get the unique identifier of this identity
    pub fn id(&self) -> Uuid {
        match self {
            Self::User(user) => user.id,
            Self::Client(client) => client.id,
        }
    }

    /// Check if this identity is a service account
    pub fn is_service_account(&self) -> bool {
        matches!(self, Self::Client(_))
    }

    /// Check if this identity is a regular user (not associated with a client)
    pub fn is_regular_user(&self) -> bool {
        matches!(self, Self::User(user) if user.client_id.is_none())
    }

    /// Get the user if this identity represents a user
    pub fn as_user(&self) -> Option<&User> {
        match self {
            Self::User(user) => Some(user),
            _ => None,
        }
    }

    /// Get the client if this identity represents a client
    pub fn as_client(&self) -> Option<&Client> {
        match self {
            Self::Client(client) => Some(client),
            _ => None,
        }
    }

    /// Get the realm ID this identity belongs to
    pub fn realm_id(&self) -> Uuid {
        match self {
            Self::User(user) => user.realm_id,
            Self::Client(client) => client.realm_id,
        }
    }

    /// Check if this identity has access to the specified realm
    ///
    /// Business rule: An identity can only access resources in its own realm
    pub fn has_access_to_realm(&self, realm_id: Uuid) -> bool {
        self.realm_id() == realm_id
    }

    /// Get a display name for this identity
    pub fn display_name(&self) -> String {
        match self {
            Self::User(user) => user.username.clone(),
            Self::Client(client) => format!("client:{}", client.client_id),
        }
    }
}
