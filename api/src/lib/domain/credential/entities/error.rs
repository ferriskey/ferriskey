use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CredentialError {
    #[error("Hash password error: {0}")]
    HashPasswordError(String),
    #[error("Verify password error: {0}")]
    VerifyPasswordError(String),
    #[error("Delete password credential error")]
    DeletePasswordCredentialError,
    #[error("Create credential error")]
    CreateCredentialError,
    #[error("Get password credential error")]
    GetPasswordCredentialError,
}
