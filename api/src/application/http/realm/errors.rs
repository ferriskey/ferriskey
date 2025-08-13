use crate::application::http::server::api_entities::api_error::ApiError;
use ferriskey_core::domain::realm::entities::RealmError;

impl From<RealmError> for ApiError {
    fn from(value: RealmError) -> Self {
        match value {
            RealmError::NotFound => ApiError::NotFound("Realm not found".to_string()),
            RealmError::AlreadyExists => ApiError::validation_error("Realm already exists", "name"),
            RealmError::Invalid => ApiError::validation_error("Realm format is invalid", "name"),
            RealmError::InternalServerError => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
            RealmError::CannotDeleteMaster => {
                ApiError::Forbidden("Cannot delete master realm".to_string())
            }
            RealmError::Forbidden => ApiError::Forbidden("Forbidden".to_string()),
        }
    }
}
