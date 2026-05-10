use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalTheme,
        ports::{PortalThemeService, UpdateThemeInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::{
    portal_theme::validators::UpdateThemeValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateThemeResponse {
    pub data: PortalTheme,
}

#[utoipa::path(
    put,
    path = "",
    tag = "portal-theme",
    summary = "Update portal theme",
    description = "Upserts the portal theme configuration for the realm. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = UpdateThemeValidator,
    responses(
        (status = 200, description = "Theme configuration updated successfully", body = UpdateThemeResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_theme(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateThemeValidator>,
) -> Result<Response<UpdateThemeResponse>, ApiError> {
    let theme = state
        .service
        .update_theme(
            identity,
            UpdateThemeInput {
                realm_name,
                config: payload.config,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateThemeResponse { data: theme }))
}
