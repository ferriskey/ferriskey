use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm_branding::{
        entities::BrandingConfig,
        ports::{GetBrandingInput, RealmBrandingService},
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
pub struct GetBrandingResponse {
    pub data: BrandingConfig,
}

#[utoipa::path(
    get,
    path = "",
    tag = "realm-branding",
    summary = "Get realm branding",
    description = "Retrieves the branding configuration for the realm. Requires manage_realm permission. Falls back to defaults when no configuration is stored.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Branding configuration retrieved successfully", body = GetBrandingResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_branding(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetBrandingResponse>, ApiError> {
    let config = state
        .service
        .get_branding(identity, GetBrandingInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetBrandingResponse { data: config }))
}
