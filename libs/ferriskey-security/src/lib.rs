use thiserror::Error;

pub mod cipher;
pub mod crypto;
pub mod jwt;

pub use cipher::field_cipher::FieldCipher;
pub use cipher::secrets::{EnvSecretsProvider, SecretsProvider};

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

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),
}
