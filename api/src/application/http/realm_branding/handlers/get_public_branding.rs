use axum::extract::{Path, State};
use ferriskey_core::domain::realm_branding::{
    entities::BrandingConfig,
    ports::{GetBrandingInput, RealmBrandingService},
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
pub struct GetPublicBrandingResponse {
    pub data: BrandingConfig,
}

#[utoipa::path(
    get,
    path = "/public",
    tag = "realm-branding",
    summary = "Get public realm branding",
    description = "Public, unauthenticated endpoint exposing the realm's branding so the auth portal can theme itself before login. Returns defaults when no configuration is stored.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Branding configuration retrieved successfully", body = GetPublicBrandingResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_public_branding(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<GetPublicBrandingResponse>, ApiError> {
    let config = state
        .service
        .get_public_branding(GetBrandingInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetPublicBrandingResponse { data: config }))
}
