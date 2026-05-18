use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalTheme,
        ports::{ListThemesInput, PortalThemeService},
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
pub struct ListThemesResponse {
    pub data: Vec<PortalTheme>,
}

#[utoipa::path(
    get,
    path = "/portal/themes",
    tag = "portal-theme",
    summary = "List portal themes",
    description = "Lists every portal theme defined in the realm. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Themes retrieved successfully", body = ListThemesResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_themes(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ListThemesResponse>, ApiError> {
    let themes = state
        .service
        .list_themes(identity, ListThemesInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ListThemesResponse { data: themes }))
}
