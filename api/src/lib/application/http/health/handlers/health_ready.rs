use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::health::entities::ReadinessResponse;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

#[derive(TypedPath)]
#[typed_path("/health/ready")]
pub struct HealthReadyRoute;

pub async fn health_ready(
    _: HealthReadyRoute,
    State(state): State<AppState>,
) -> Result<Response<ReadinessResponse>, ApiError> {
    let readiness_response = state
        .use_case_bundle
        .health_check_use_case
        .execute_readiness()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Response::OK(readiness_response))
}