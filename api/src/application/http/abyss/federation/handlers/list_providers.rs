use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::{entities::FederationProvider, ports::FederationService},
    authentication::value_objects::Identity,
};

use crate::application::http::{
    abyss::federation::dto::ListProvidersResponse,
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
    path = "/federation/providers",
    summary = "List federation providers in a realm",
    responses(
        (status = 200, description = "List of providers", body = ListProvidersResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    tag = "federation"
)]
pub async fn list_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ListProvidersResponse>, ApiError> {
    let providers = state
        .service
        .list_federation_providers(identity, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ListProvidersResponse {
        data: providers
            .into_iter()
            .map(FederationProvider::into)
            .collect(),
    }))
}
