use ferriskey_core::domain::common::entities::app_errors::CoreError;

use crate::application::http::server::api_entities::api_error::ApiError;

impl From<CoreError> for ApiError {
    fn from(_error: CoreError) -> Self {
        todo!()
    }
}
