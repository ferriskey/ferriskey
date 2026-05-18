use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::ports::{GetThemeByIdInput, PortalThemeService},
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActivateThemeResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/{theme_id}/activate",
    tag = "portal-theme",
    summary = "Activate a portal theme",
    description = "Marks the given theme as the realm's active theme. Stored in realm_settings.portal_theme_id.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
    ),
    responses(
        (status = 200, description = "Theme activated successfully", body = ActivateThemeResponse),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn activate_theme(
    Path((realm_name, theme_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ActivateThemeResponse>, ApiError> {
    state
        .service
        .activate_theme(
            identity,
            GetThemeByIdInput {
                realm_name,
                theme_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ActivateThemeResponse {
        message: "Portal theme activated successfully".to_string(),
    }))
}
