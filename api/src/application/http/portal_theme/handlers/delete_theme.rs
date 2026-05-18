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
pub struct DeleteThemeResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/portal/themes/{theme_id}",
    tag = "portal-theme",
    summary = "Delete a portal theme",
    description = "Deletes a portal theme. Refuses (400) when the theme is currently active for the realm.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
    ),
    responses(
        (status = 200, description = "Theme deleted successfully", body = DeleteThemeResponse),
        (status = 400, description = "Theme is currently active", body = ApiErrorResponse),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_theme(
    Path((realm_name, theme_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteThemeResponse>, ApiError> {
    state
        .service
        .delete_theme(
            identity,
            GetThemeByIdInput {
                realm_name,
                theme_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteThemeResponse {
        message: "Portal theme deleted successfully".to_string(),
    }))
}
