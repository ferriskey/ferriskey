use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::ports::FederationService, authentication::value_objects::Identity,
};
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::ProviderResponse,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse},
            response::Response,
        },
        app_state::AppState,
    },
};

#[utoipa::path(
    get,
    path = "/federation/providers/{id}",
    summary = "Get a federation provider details by ID",
    responses(
        (status = 200, description = "Provider details", body = ProviderResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm or Provider not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "federation"
)]
pub async fn get_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProviderResponse>, ApiError> {
    let provider = state
        .service
        .get_federation_provider(identity, id, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(provider.into()))
}
