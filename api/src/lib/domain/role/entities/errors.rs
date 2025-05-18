use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RoleError {
    #[error("Role not found")]
    NotFound,
    #[error("Role already exists")]
    AlreadyExists,
    #[error("Invalid role")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Forbidden")]
    Forbidden,
}
