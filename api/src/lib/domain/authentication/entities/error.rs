use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AuthenticationError {
    #[error("Token not found")]
    NotFound,

    #[error("Invalid client")]
    Invalid,

    #[error("Internal server error")]
    InternalServerError,
}
