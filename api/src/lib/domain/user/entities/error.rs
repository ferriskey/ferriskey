use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Invalid user")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden: {0}")]
    Forbidden(String),
}
