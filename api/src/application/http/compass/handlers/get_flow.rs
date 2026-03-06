use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    compass::{entities::CompassFlow, ports::CompassService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetFlowResponse {
    data: CompassFlow,
}

#[derive(Debug, Deserialize)]
pub struct FlowPathParams {
    pub realm_name: String,
    pub flow_id: Uuid,
}

#[utoipa::path(
    get,
    summary = "Get Compass Flow",
    path = "/compass/v1/flows/{flow_id}",
    tag = "compass",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("flow_id" = Uuid, Path, description = "Flow ID"),
    ),
    responses(
        (status = 200, description = "Flow retrieved successfully", body = GetFlowResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Flow not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_flow(
    Path(params): Path<FlowPathParams>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetFlowResponse>, ApiError> {
    let flow = state
        .service
        .get_flow(identity, params.realm_name, params.flow_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetFlowResponse { data: flow }))
}
