use thiserror::Error;

pub mod crypto;
pub mod jwt;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Hashing error: {0}")]
    HashingError(String),
    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Invalid key: {details}")]
    InvalidKey { details: String },
}
