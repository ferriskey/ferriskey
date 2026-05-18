use axum::extract::{Path, Query, State};
use ferriskey_core::domain::portal_theme::{
    entities::{PortalPageType, PortalThemeConfig},
    ports::{ListThemesInput, PortalThemeService},
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

#[derive(Debug, Deserialize)]
pub struct ActiveThemeQuery {
    pub page_type: PortalPageType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ActiveThemeResponse {
    pub design_tokens: PortalThemeConfig,
    pub layout_id: Option<Uuid>,
    pub page_tree: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/active",
    tag = "portal-theme-public",
    summary = "Get the active portal theme bundle for a page",
    description = "Public endpoint used by the portal renderer. Returns the realm's active theme design tokens, the referenced layout ID (if any), and the JSONB component tree for the requested page type. Callers fetch the layout itself via the public portal-layouts endpoint. Falls back to defaults / empty trees when nothing is configured.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("page_type" = PortalPageType, Query, description = "Portal page type"),
    ),
    responses(
        (status = 200, description = "Active theme bundle retrieved successfully", body = ActiveThemeResponse),
        (status = 401, description = "Unknown realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_active_theme(
    Path(realm_name): Path<String>,
    Query(query): Query<ActiveThemeQuery>,
    State(state): State<AppState>,
) -> Result<Response<ActiveThemeResponse>, ApiError> {
    let active = state
        .service
        .get_active_theme(ListThemesInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    let (design_tokens, layout_id, page_tree) = match active {
        Some(theme) => (
            theme.config,
            theme.layout_id,
            theme.pages.get(query.page_type).clone(),
        ),
        None => (
            PortalThemeConfig::default(),
            None,
            serde_json::Value::Array(vec![]),
        ),
    };

    Ok(Response::OK(ActiveThemeResponse {
        design_tokens,
        layout_id,
        page_tree,
    }))
}
