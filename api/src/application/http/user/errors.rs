use crate::application::http::server::api_entities::api_error::ApiError;
use ferriskey_core::domain::credential::entities::CredentialError;

impl From<CredentialError> for ApiError {
    fn from(value: CredentialError) -> Self {
        match value {
            CredentialError::CreateCredentialError => {
                ApiError::InternalServerError("Failed to create credential".into())
            }
            CredentialError::GetUserCredentialsError => {
                ApiError::InternalServerError("Failed to get credential".into())
            }
            CredentialError::DeleteCredentialError => {
                ApiError::InternalServerError("Failed to delete credential".into())
            }
            CredentialError::VerifyPasswordError(error) => {
                ApiError::InternalServerError(format!("Failed to verify password: {error}").into())
            }
            CredentialError::DeletePasswordCredentialError => {
                ApiError::InternalServerError("Failed to delete password credential".into())
            }
            CredentialError::GetPasswordCredentialError => {
                ApiError::InternalServerError("Failed to get password credential".into())
            }
            CredentialError::HashPasswordError(error) => {
                ApiError::InternalServerError(format!("Failed to hash password: {error}").into())
            }
            CredentialError::UpdateCredentialError => {
                ApiError::InternalServerError("Internal server error".into())
            }
            CredentialError::UnexpectedCredentialData => {
                ApiError::InternalServerError("Internal server error".into())
            }
        }
    }
}
