use crate::application::http::server::api_entities::api_error::ApiError;
use ferriskey_core::domain::client::entities::ClientError;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUriError;

impl From<ClientError> for ApiError {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            ClientError::NotFound => ApiError::NotFound("Client not found".to_string()),
            ClientError::Invalid => ApiError::validation_error("Client format is invalid", "name"),
            ClientError::AlreadyExists => {
                ApiError::validation_error("Client already exists", "name")
            }
            ClientError::RedirectUriNotFound => {
                ApiError::validation_error("Redirect URI not found", "redirect_uri")
            }
            ClientError::InvalidRedirectUri => {
                ApiError::validation_error("Redirect URI format is invalid", "redirect_uri")
            }
            ClientError::Forbidden(e) => ApiError::Forbidden(e),
        }
    }
}

impl From<RedirectUriError> for ApiError {
    fn from(value: RedirectUriError) -> Self {
        match value {
            RedirectUriError::NotFound => ApiError::NotFound("Redirect URI not found".to_string()),
            RedirectUriError::AlreadyExists => {
                ApiError::validation_error("Redirect URI already exists", "redirect_uri")
            }
            RedirectUriError::Invalid => {
                ApiError::validation_error("Redirect URI format is invalid", "redirect_uri")
            }
            RedirectUriError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            RedirectUriError::DatabaseError => {
                ApiError::InternalServerError("Database error".to_string())
            }
            RedirectUriError::NotEnabled => {
                ApiError::validation_error("Redirect URI not enabled", "redirect_uri")
            }
            RedirectUriError::NotValid => {
                ApiError::validation_error("Redirect URI not valid", "redirect_uri")
            }
        }
    }
}
