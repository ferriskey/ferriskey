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
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::ProviderResponse, server::api_entities::api_error::ApiError,
    server::app_state::AppState,
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
) -> Result<(StatusCode, Json<ProviderResponse>), ApiError> {
    // 1. Verify realm access
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    // 2. Get provider
    let provider = state
        .service
        .get_federation_provider(id)
        .await
        .map_err(ApiError::from)?;

    // 3. Verify provider belongs to realm
    if provider.realm_id != Into::<Uuid>::into(realm.id) {
        return Err(ApiError::NotFound(
            "Provider not found in this realm".to_string(),
        ));
    }

    Ok((StatusCode::OK, Json(ProviderResponse::from(provider))))
}
