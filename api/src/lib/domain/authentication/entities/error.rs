use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AuthenticationError {
    #[error("Token not found")]
    NotFound,

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

    #[error("Internal server error")]
    InternalServerError,
}
