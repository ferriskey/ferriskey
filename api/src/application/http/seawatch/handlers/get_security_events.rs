use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetSecurityEventsResponse {}

pub async fn get_security_events(
    #[allow(unused)] State(state): State<AppState>,
) -> Result<Response<GetSecurityEventsResponse>, ApiError> {
    todo!()
}
