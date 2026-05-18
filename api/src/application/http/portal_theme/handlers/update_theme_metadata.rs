use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalTheme,
        ports::{PortalThemeService, UpdateThemeMetadataInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::{
    portal_theme::validators::UpdateThemeMetadataValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateThemeMetadataResponse {
    pub data: PortalTheme,
}

#[utoipa::path(
    put,
    path = "/{theme_id}",
    tag = "portal-theme",
    summary = "Update portal theme metadata",
    description = "Updates a portal theme's name, layout reference, and design tokens. Page trees are managed via /pages/{page_type}.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
    ),
    request_body = UpdateThemeMetadataValidator,
    responses(
        (status = 200, description = "Theme updated successfully", body = UpdateThemeMetadataResponse),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_theme_metadata(
    Path((realm_name, theme_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateThemeMetadataValidator>,
) -> Result<Response<UpdateThemeMetadataResponse>, ApiError> {
    let theme = state
        .service
        .update_theme_metadata(
            identity,
            UpdateThemeMetadataInput {
                realm_name,
                theme_id,
                name: payload.name,
                layout_id: payload.layout_id,
                config: payload.config,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateThemeMetadataResponse {
        data: theme,
    }))
}
