use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm_branding::{
        entities::RealmBranding,
        ports::{RealmBrandingService, UpdateBrandingInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::{
    realm_branding::validators::UpdateBrandingValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateBrandingResponse {
    pub data: RealmBranding,
}

#[utoipa::path(
    put,
    path = "",
    tag = "realm-branding",
    summary = "Update realm branding",
    description = "Upserts the branding configuration for the realm. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = UpdateBrandingValidator,
    responses(
        (status = 200, description = "Branding configuration updated successfully", body = UpdateBrandingResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_branding(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateBrandingValidator>,
) -> Result<Response<UpdateBrandingResponse>, ApiError> {
    let branding = state
        .service
        .update_branding(
            identity,
            UpdateBrandingInput {
                realm_name,
                config: payload.config,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateBrandingResponse { data: branding }))
}
