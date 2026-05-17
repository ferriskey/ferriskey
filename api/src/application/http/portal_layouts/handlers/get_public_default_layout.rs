use axum::extract::{Path, State};
use ferriskey_core::domain::portal_layouts::{
    entities::PortalLayout,
    ports::{ListLayoutsInput, PortalLayoutsService},
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
pub struct GetPublicDefaultPortalLayoutResponse {
    pub data: Option<PortalLayout>,
}

#[utoipa::path(
    get,
    path = "/default",
    tag = "portal-layouts",
    summary = "Get the public default portal layout",
    description = "Returns the realm's default portal layout for the unauthenticated portal pages. No auth required.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Default layout retrieved (may be null)", body = GetPublicDefaultPortalLayoutResponse),
        (status = 401, description = "Unknown realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_public_default_layout(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<GetPublicDefaultPortalLayoutResponse>, ApiError> {
    let layout = state
        .service
        .get_public_default_layout(ListLayoutsInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetPublicDefaultPortalLayoutResponse {
        data: layout,
    }))
}
