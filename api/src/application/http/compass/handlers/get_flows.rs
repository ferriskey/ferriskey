use axum::{
    Extension,
    extract::{Path, Query, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    compass::{entities::CompassFlow, ports::CompassService, value_objects::FetchFlowsInput},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetFlowsResponse {
    data: Vec<CompassFlow>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetFlowsQuery {
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[utoipa::path(
    get,
    summary = "Get Compass Flows",
    path = "/compass/v1/flows",
    tag = "compass",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetFlowsQuery,
    ),
    responses(
        (status = 200, description = "Flows retrieved successfully", body = GetFlowsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_flows(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<GetFlowsQuery>,
) -> Result<Response<GetFlowsResponse>, ApiError> {
    let filter = ferriskey_core::domain::compass::value_objects::FlowFilter {
        client_id: query.client_id,
        user_id: query.user_id,
        grant_type: query.grant_type,
        status: query.status,
        from_timestamp: None,
        to_timestamp: None,
        limit: query.limit.or(Some(50)),
        offset: query.offset.or(Some(0)),
    };

    let flows = state
        .service
        .fetch_flows(identity, FetchFlowsInput { realm_name, filter })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetFlowsResponse { data: flows }))
}
