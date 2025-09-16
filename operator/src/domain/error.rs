use thiserror::Error;

#[derive(Debug, Error)]
pub enum OperatorError {
    #[error("Failed to delete cluster {message}")]
    DeleteClusterError { message: String },
    #[error("Internal server error {message}")]
    InternalServerError { message: String },
    #[error("Failed to apply API resources: {message}")]
    ApplyApiError { message: String },
    #[error("Failed to delete API resources: {message}")]
    DeleteApiError { message: String },
    #[error("Invalid specification: {message}")]
    InvalidSpec { message: String },
}
