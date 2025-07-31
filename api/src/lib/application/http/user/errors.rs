use crate::application::http::server::api_entities::api_error::ApiError;
use ferriskey_core::domain::credential::entities::CredentialError;
use ferriskey_core::domain::user::entities::UserError;

impl From<UserError> for ApiError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::AlreadyExists => {
                ApiError::validation_error("User already exists", "username")
            }
            UserError::NotFound => ApiError::NotFound("User not found".to_string()),
            UserError::Invalid => ApiError::validation_error("User is invalid", "username"),
            UserError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            UserError::Forbidden(message) => ApiError::Forbidden(message),
        }
    }
}

impl From<CredentialError> for ApiError {
    fn from(value: CredentialError) -> Self {
        match value {
            CredentialError::CreateCredentialError => {
                ApiError::InternalServerError("Failed to create credential".to_string())
            }
            CredentialError::GetUserCredentialsError => {
                ApiError::InternalServerError("Failed to get credential".to_string())
            }
            CredentialError::DeleteCredentialError => {
                ApiError::InternalServerError("Failed to delete credential".to_string())
            }
            CredentialError::VerifyPasswordError(error) => {
                ApiError::InternalServerError(format!("Failed to verify password: {error}"))
            }
            CredentialError::DeletePasswordCredentialError => {
                ApiError::InternalServerError("Failed to delete password credential".to_string())
            }
            CredentialError::GetPasswordCredentialError => {
                ApiError::InternalServerError("Failed to get password credential".to_string())
            }
            CredentialError::HashPasswordError(error) => {
                ApiError::InternalServerError(format!("Failed to hash password: {error}"))
            }
        }
    }
}
