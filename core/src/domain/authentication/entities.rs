use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

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
