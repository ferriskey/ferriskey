use crate::{
    application::http::server::api_entities::api_error::ApiError,
    domain::client::entities::error::ClientError,
};

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
        }
    }
}
