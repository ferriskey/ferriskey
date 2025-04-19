use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RedirectUriError {
    #[error("Redirect URI not found")]
    NotFound,

    #[error("Redirect URI already exists")]
    AlreadyExists,

    #[error("Invalid redirect URI")]
    Invalid,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Database error")]
    DatabaseError,

    #[error("Redirect URI not enabled")]
    NotEnabled,

    #[error("Redirect URI not valid")]
    NotValid,
}
