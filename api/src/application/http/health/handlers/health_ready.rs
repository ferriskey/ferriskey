use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use chrono::Utc;
use ferriskey_core::domain::health::entities::ReadinessResponse;
use ferriskey_core::domain::health::ports::HealthCheckService;

pub async fn health_ready(
    State(state): State<AppState>,
) -> Result<Response<ReadinessResponse>, ApiError> {
    let readness = state
        .service
        .readness()
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?;

    let status = readness.status.clone();
    let overall_status = match status.as_str() {
        "ok" | "healthy" => "ok".to_string(),
        _ => "unhealthy".to_string(),
    };

    let is_healthy = overall_status == "ok";

    Ok(Response::OK(ReadinessResponse {
        is_healthy,
        status,
        database: readness,
        timestamp: Utc::now().to_rfc3339(),
    }))
}
