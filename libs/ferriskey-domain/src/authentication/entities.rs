use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct JwtToken {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    id_token: Option<String>,
}

impl JwtToken {
    pub fn new(
        access_token: String,
        token_type: String,
        refresh_token: String,
        expires_in: u32,
        id_token: Option<String>,
    ) -> Self {
        Self {
            access_token,
            token_type,
            refresh_token,
            expires_in,
            id_token,
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
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
