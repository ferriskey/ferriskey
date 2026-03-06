use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    compass::{ports::CompassService, value_objects::FlowStats},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetStatsResponse {
    data: FlowStats,
}

#[utoipa::path(
    get,
    summary = "Get Compass Stats",
    path = "/compass/v1/stats",
    tag = "compass",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Stats retrieved successfully", body = GetStatsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_stats(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetStatsResponse>, ApiError> {
    let stats = state
        .service
        .get_stats(identity, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetStatsResponse { data: stats }))
}
