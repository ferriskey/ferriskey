use axum::extract::{Path, State};
use ferriskey_core::domain::portal_layouts::{
    entities::PortalLayout,
    ports::{GetLayoutInput, PortalLayoutsService},
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
pub struct GetPublicPortalLayoutResponse {
    pub data: Option<PortalLayout>,
}

#[utoipa::path(
    get,
    path = "/{layout_id}",
    tag = "portal-layouts",
    summary = "Get a portal layout by id (public)",
    description = "Returns a specific portal layout without auth, used by the portal renderer to fetch the layout referenced by the active theme.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("layout_id" = Uuid, Path, description = "Portal layout ID"),
    ),
    responses(
        (status = 200, description = "Layout retrieved (may be null)", body = GetPublicPortalLayoutResponse),
        (status = 401, description = "Unknown realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_public_layout(
    Path((realm_name, layout_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
) -> Result<Response<GetPublicPortalLayoutResponse>, ApiError> {
    let layout = state
        .service
        .get_public_layout(GetLayoutInput {
            realm_name,
            layout_id,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetPublicPortalLayoutResponse { data: layout }))
}
