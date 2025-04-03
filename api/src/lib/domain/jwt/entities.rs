use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: i64,
    pub iss: String,
    pub aud: Option<String>,
    pub realm: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct Jwt {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token generation error: {0}")]
    GenerationError(String),

    #[error("Token validation error: {0}")]
    ValidationError(String),

    #[error("Token parsing error: {0}")]
    ParsingError(String),

    #[error("Token expiration error: {0}")]
    ExpirationError(String),

    #[error("Realm key not found")]
    RealmKeyNotFound,

    #[error("Invalid token")]
    InvalidToken,
}