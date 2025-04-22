use thiserror::Error;

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
