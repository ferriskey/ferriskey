use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Internal server error")]
    InternalServerError,
}
