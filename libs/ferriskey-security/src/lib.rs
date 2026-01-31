use thiserror::Error;

pub mod crypto;
pub mod jwt;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Hashing error: {0}")]
    HashingError(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

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

    #[error("Expired token")]
    ExpiredToken,
}
