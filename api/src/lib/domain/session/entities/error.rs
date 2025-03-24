use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Session is invalid")]
    Invalid,
    #[error("Failed to create session")]
    CreateError,
    #[error("Failed to delete session")]
    DeleteError,
}
