use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalThemeConfig,
        ports::{GetThemeInput, PortalThemeService},
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

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetThemeResponse {
    pub data: PortalThemeConfig,
}

#[utoipa::path(
    get,
    path = "",
    tag = "portal-theme",
    summary = "Get portal theme",
    description = "Retrieves the portal theme configuration for the realm. Requires manage_realm permission. Falls back to defaults when no configuration is stored.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Theme configuration retrieved successfully", body = GetThemeResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_theme(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetThemeResponse>, ApiError> {
    let config = state
        .service
        .get_theme(identity, GetThemeInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetThemeResponse { data: config }))
}
