use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ClientError {
    #[error("Client not found")]
    NotFound,
    #[error("Client already exists")]
    AlreadyExists,
    #[error("Invalid client")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Redirect URI not found")]
    RedirectUriNotFound,
    #[error("Invalid redirect URI")]
    InvalidRedirectUri,
}
