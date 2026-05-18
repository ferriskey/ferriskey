use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::{
        entities::PortalTheme,
        ports::{CreateThemeInput, PortalThemeService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::{
    portal_theme::validators::CreateThemeValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateThemeResponse {
    pub data: PortalTheme,
}

#[utoipa::path(
    post,
    path = "/portal/themes",
    tag = "portal-theme",
    summary = "Create a portal theme",
    description = "Creates a new portal theme in the realm with empty page trees. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = CreateThemeValidator,
    responses(
        (status = 201, description = "Theme created successfully", body = CreateThemeResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn create_theme(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateThemeValidator>,
) -> Result<Response<CreateThemeResponse>, ApiError> {
    let theme = state
        .service
        .create_theme(
            identity,
            CreateThemeInput {
                realm_name,
                name: payload.name,
                layout_id: payload.layout_id,
                config: payload.config,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(CreateThemeResponse { data: theme }))
}
