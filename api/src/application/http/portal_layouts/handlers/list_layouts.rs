use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::{
        entities::PortalLayout,
        ports::{ListLayoutsInput, PortalLayoutsService},
    },
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListPortalLayoutsResponse {
    pub data: Vec<PortalLayout>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "portal-layouts",
    summary = "List portal layouts",
    description = "Returns every portal layout configured for the realm.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Layouts retrieved successfully", body = ListPortalLayoutsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_layouts(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ListPortalLayoutsResponse>, ApiError> {
    let layouts = state
        .service
        .list_layouts(identity, ListLayoutsInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ListPortalLayoutsResponse { data: layouts }))
}
