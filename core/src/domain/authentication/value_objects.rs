use enum_display::EnumDisplay;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::authentication::entities::DecodedToken;
use crate::domain::realm::entities::RealmId;
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
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
}

pub struct GrantTypeParams {
    pub realm_id: RealmId,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserInput {
    pub realm_name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl CreateAuthSessionRequest {
    pub fn new(realm_id: Uuid, client_id: Uuid, redirect_uri: String) -> Self {
        Self {
            realm_id,
            client_id,
            redirect_uri,
            response_type: "code".to_string(),
            scope: Some("openid".to_string()),
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
        self.scope = Some(scope);
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
    pub fn realm_id(&self) -> RealmId {
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

    /// Get the kind of this identity
    pub fn kind(&self) -> IdentityKind {
        match self {
            Self::User(_) => IdentityKind::User,
            Self::Client(_) => IdentityKind::Client,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumDisplay, Eq, PartialEq)]
#[display(case = "Kebab")]
pub enum IdentityKind {
    User,
    Client,
}

pub struct GenerateTokenInput {
    pub base_url: String,
    pub realm_name: String,
    pub user_id: Uuid,
    pub username: String,
    pub client_id: String,
    pub email: String,
    pub realm_id: RealmId,
    pub scope: Option<String>,
}

pub struct GetUserInfoInput {
    pub realm_name: String,
    pub identity: Identity,
    pub token: DecodedToken,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UserInfoResponse {
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
}

impl UserInfoResponse {
    pub fn from_user(user: &User) -> Self {
        Self {
            sub: user.id.to_string(),
            email: Some(user.email.to_string()),
            email_verified: Some(user.email_verified),
            family_name: Some(user.firstname.to_string()),
            given_name: Some(user.lastname.to_string()),
            name: Some(format!("{} {}", user.firstname, user.lastname)),
            preferred_username: Some(user.username.to_string()),
        }
    }
}
