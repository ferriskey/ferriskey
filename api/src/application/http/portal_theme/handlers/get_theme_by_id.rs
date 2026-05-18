use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalTheme,
        ports::{GetThemeByIdInput, PortalThemeService},
    },
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
pub struct GetThemeByIdResponse {
    pub data: PortalTheme,
}

#[utoipa::path(
    get,
    path = "/portal/themes/{theme_id}",
    tag = "portal-theme",
    summary = "Get a portal theme",
    description = "Retrieves a single portal theme, including design tokens, layout reference, and all seven page trees.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
    ),
    responses(
        (status = 200, description = "Theme retrieved successfully", body = GetThemeByIdResponse),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_theme_by_id(
    Path((realm_name, theme_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetThemeByIdResponse>, ApiError> {
    let theme = state
        .service
        .get_theme_by_id(
            identity,
            GetThemeByIdInput {
                realm_name,
                theme_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetThemeByIdResponse { data: theme }))
}
