use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::{
    abyss::federation::ports::FederationService,
    authentication::value_objects::Identity,
    realm::ports::{GetRealmInput, RealmService},
};

use crate::application::http::{
    abyss::federation::dto::ProviderResponse, server::api_entities::api_error::ApiError,
    server::app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/federation/providers",
    responses(
        (status = 200, description = "List of providers", body = Vec<ProviderResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
    ),
    params(
        ("realm" = String, Path, description = "Realm name")
    ),
    tag = "Federation"
)]
pub async fn list_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<(StatusCode, Json<Vec<ProviderResponse>>), ApiError> {
    // 1. Get realm
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    // 2. List providers
    let providers = state
        .service
        .list_federation_providers(realm.id.into())
        .await
        .map_err(ApiError::from)?;

    // 3. Map to DTOs
    let response: Vec<ProviderResponse> =
        providers.into_iter().map(ProviderResponse::from).collect();

    Ok((StatusCode::OK, Json(response)))
}
