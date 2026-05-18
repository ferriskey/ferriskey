use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::{PortalPageType, PortalTheme},
        ports::{PortalThemeService, UpdateThemePageInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::{
    portal_theme::validators::UpdateThemePageValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateThemePageResponse {
    pub data: PortalTheme,
}

#[utoipa::path(
    put,
    path = "/portal/themes/{theme_id}/pages/{page_type}",
    tag = "portal-theme",
    summary = "Update a portal page tree",
    description = "Replaces the JSONB component tree for a single page type. The submitted tree must contain every required block type for the page; otherwise 422 is returned.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
        ("page_type" = PortalPageType, Path, description = "Portal page type"),
    ),
    request_body = UpdateThemePageValidator,
    responses(
        (status = 200, description = "Page updated successfully", body = UpdateThemePageResponse),
        (status = 422, description = "Tree is missing required block types", body = ApiErrorResponse),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_theme_page(
    Path((realm_name, theme_id, page_type)): Path<(String, Uuid, PortalPageType)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateThemePageValidator>,
) -> Result<Response<UpdateThemePageResponse>, ApiError> {
    let theme = state
        .service
        .update_theme_page(
            identity,
            UpdateThemePageInput {
                realm_name,
                theme_id,
                page_type,
                tree: payload.tree,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateThemePageResponse { data: theme }))
}
