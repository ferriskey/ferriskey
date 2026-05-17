use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::{
        entities::PortalLayout,
        ports::{GetLayoutInput, PortalLayoutsService},
    },
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetPortalLayoutResponse {
    pub data: PortalLayout,
}

#[utoipa::path(
    get,
    path = "/{layout_id}",
    tag = "portal-layouts",
    summary = "Get a portal layout",
    description = "Retrieves a single portal layout by id.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("layout_id" = Uuid, Path, description = "Portal layout ID"),
    ),
    responses(
        (status = 200, description = "Layout retrieved successfully", body = GetPortalLayoutResponse),
        (status = 404, description = "Layout not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_layout(
    Path((realm_name, layout_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetPortalLayoutResponse>, ApiError> {
    let layout = state
        .service
        .get_layout(
            identity,
            GetLayoutInput {
                realm_name,
                layout_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetPortalLayoutResponse { data: layout }))
}
