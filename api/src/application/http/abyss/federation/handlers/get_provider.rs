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
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[utoipa::path(
    get,
    path = "/federation/providers/{id}",
    summary = "Get Federation Provider Details",
    responses(
        (status = 200, description = "Provider details", body = ProviderResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm or Provider not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "Federation"
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
