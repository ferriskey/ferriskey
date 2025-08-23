use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use ferriskey_core::domain::health::entities::ReadinessResponse;

pub async fn health_ready(
    State(state): State<AppState>,
) -> Result<Response<ReadinessResponse>, ApiError> {
    let readiness_response = state
        .use_case_bundle
        .health_check_use_case
        .execute_readiness()
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?;

    if !readiness_response.is_healthy {
        return Err(ApiError::ServiceUnavailable(
            "Service is not ready".to_string(),
        ));
    }

    Ok(Response::OK(readiness_response))
}
