use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity, common::generate_timestamp, jwt::entities::JwtClaim,
    user::entities::RequiredAction,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct JwtToken {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u32,
    id_token: String,
}

impl JwtToken {
    pub fn new(
        access_token: String,
        token_type: String,
        refresh_token: String,
        expires_in: u32,
        id_token: String,
    ) -> Self {
        Self {
            access_token,
            token_type,
            refresh_token,
            expires_in,
            id_token,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
}

impl RefreshClaims {
    pub fn new(sub: String, exp: usize, jti: String) -> Self {
        Self { sub, exp, jti }
    }
}

#[derive(Debug, Clone, Error)]
pub enum AuthenticationError {
    #[error("Token not found")]
    NotFound,

    #[error("Service account not found")]
    ServiceAccountNotFound,

    #[error("Invalid client")]
    Invalid,

    #[error("Invalid realm")]
    InvalidRealm,

    #[error("Invalid client")]
    InvalidClient,

    #[error("Invalid user")]
    InvalidUser,

    #[error("Password is invalid")]
    InvalidPassword,

    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Invalid authorization request")]
    InvalidRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub code: Option<String>,
    pub authenticated: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthSessionParams {
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub code: Option<String>,
    pub authenticated: bool,
}

impl AuthSession {
    pub fn new(params: AuthSessionParams) -> Self {
        let now = Utc::now();
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: params.realm_id,
            client_id: params.client_id,
            redirect_uri: params.redirect_uri,
            response_type: params.response_type,
            scope: params.scope,
            state: params.state,
            nonce: params.nonce,
            user_id: params.user_id,
            code: params.code,
            authenticated: params.authenticated,
            created_at: now,
            expires_at: now + Duration::minutes(10),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum GrantType {
    #[default]
    #[serde(rename = "authorization_code")]
    Code,

    #[serde(rename = "password")]
    Password,

    #[serde(rename = "client_credentials")]
    Credentials,

    #[serde(rename = "refresh_token")]
    RefreshToken,
}

impl Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantType::Code => write!(f, "code"),
            GrantType::Password => write!(f, "password"),
            GrantType::Credentials => write!(f, "credentials"),
            GrantType::RefreshToken => write!(f, "refresh_token"),
        }
    }
}

pub struct AuthInput {
    pub client_id: String,
    pub realm_name: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
}

pub struct AuthOutput {
    pub login_url: String,
    pub session: AuthSession,
}

pub struct ExchangeTokenInput {
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub base_url: String,
    pub grant_type: GrantType,
}

pub struct AuthorizeRequestInput {
    pub claims: JwtClaim,
    pub token: String,
}

pub struct AuthorizeRequestOutput {
    pub identity: Identity,
}

pub struct AuthenticateInput {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub auth_method: AuthenticationMethod,
}

impl AuthenticateInput {
    pub fn with_user_credentials(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::UserCredentials { username, password },
        }
    }

    pub fn with_existing_token(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        token: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::ExistingToken { token },
        }
    }

    pub fn is_token_refresh(&self) -> bool {
        matches!(self.auth_method, AuthenticationMethod::ExistingToken { .. })
    }

    pub fn is_credential_auth(&self) -> bool {
        matches!(
            self.auth_method,
            AuthenticationMethod::UserCredentials { .. }
        )
    }
}

pub struct AuthenticateOutput {
    pub user_id: Uuid,
    pub status: AuthenticationStepStatus,
    pub authorization_code: Option<String>,
    pub temporary_token: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub redirect_url: Option<String>,
    pub session_state: Option<String>,
}

impl AuthenticateOutput {
    pub fn complete_with_redirect(
        user_id: Uuid,
        authorization_code: String,
        redirect_url: String,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::Success,
            authorization_code: Some(authorization_code),
            temporary_token: None,
            required_actions: Vec::new(),
            redirect_url: Some(redirect_url),
            session_state: None,
        }
    }

    pub fn requires_actions(
        user_id: Uuid,
        required_actions: Vec<RequiredAction>,
        temporary_token: String,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresActions,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions,
            redirect_url: None,
            session_state: None,
        }
    }

    pub fn requires_otp_challenge(user_id: Uuid, temporary_token: String) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresOtpChallenge,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions: Vec::new(),
            redirect_url: None,
            session_state: None,
        }
    }
}

#[derive(Debug)]
pub struct CredentialsAuthParams {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationStepStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    UserCredentials { username: String, password: String },
    ExistingToken { token: String },
}
