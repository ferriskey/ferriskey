use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HealthLiveResponse {
    pub status: String,
    pub timestamp: String,
    pub message: String,
}

pub async fn health_live(
    State(state): State<AppState>,
) -> Result<Response<HealthLiveResponse>, ApiError> {
    state
        .use_case_bundle
        .health_check_use_case
        .execute_liveness()
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?;

    Ok(Response::OK(HealthLiveResponse {
        message: "Service is live".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        status: "ok".to_string(),
    }))
}
