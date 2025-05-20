use crate::{
    application::http::server::api_entities::api_error::ApiError,
    domain::role::entities::errors::RoleError,
};

impl From<RoleError> for ApiError {
    fn from(value: RoleError) -> Self {
        match value {
            RoleError::NotFound => ApiError::NotFound("Role not found".to_string()),
            RoleError::AlreadyExists => ApiError::validation_error("Role already exists", "name"),
            RoleError::Invalid => ApiError::validation_error("Role format is invalid", "name"),
            RoleError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            RoleError::Forbidden => ApiError::Forbidden("Forbidden".to_string()),
        }
    }
}
