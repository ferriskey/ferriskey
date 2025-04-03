use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: i64,
    pub iss: String,
    pub aud: Vec<String>,
    pub typ: String,
    pub azp: String,
}

// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
// pub struct Key {
//     pub kid: String,
//     pub kty: String,
//     pub alg: String,
//     #[serde(rename = "use")]
//     pub _use: String,
//     pub n: String,
//     pub e: String,
//     pub x5c: Vec<String>,
//     pub x5t: String,
// }

impl JwtClaims {
    pub fn new(sub: String, iss: String, aud: Vec<String>, typ: String, azp: String) -> Self {
        Self {
            sub,
            exp: chrono::Utc::now().timestamp() + 3600,
            iss,
            aud,
            typ,
            azp,
        }
    }
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