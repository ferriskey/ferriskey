use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::{entities::FederationProvider, ports::FederationService},
    authentication::value_objects::Identity,
};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/federation/providers",
    summary = "List federation providers in a realm",
    responses(
        (status = 200, description = "List of providers", body = Vec<FederationProvider>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    tag = "Federation"
)]
pub async fn list_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<FederationProvider>>, ApiError> {
    let providers = state
        .service
        .list_federation_providers(identity, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(providers))
}
