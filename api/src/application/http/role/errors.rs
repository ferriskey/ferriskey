use crate::application::http::server::api_entities::api_error::ApiError;
use ferriskey_core::domain::role::entities::RoleError;

impl From<RoleError> for ApiError {
    fn from(value: RoleError) -> Self {
        match value {
            RoleError::NotFound => ApiError::NotFound("Role not found".to_string()),
            RoleError::AlreadyExists => ApiError::validation_error("Role already exists", "name"),
            RoleError::Invalid => ApiError::validation_error("Role format is invalid", "name"),
            RoleError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            RoleError::Forbidden(e) => ApiError::Forbidden(e),
            RoleError::FailedWebhookNotification(e) => {
                ApiError::InternalServerError(format!("Failed to send webhook notification: {}", e))
            }
        }
    }
}
