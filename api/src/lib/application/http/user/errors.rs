use crate::{
    application::http::server::api_entities::api_error::ApiError,
    domain::user::entities::error::UserError,
};

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
        }
    }
}
