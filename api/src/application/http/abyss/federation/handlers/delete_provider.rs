use axum::{
    Extension,
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
    server::api_entities::api_error::ApiError, server::app_state::AppState,
};

#[utoipa::path(
    delete,
    path = "/federation/providers/{id}",
    responses(
        (status = 204, description = "Provider deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm or Provider not found"),
    ),
    params(
        ("realm" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "Federation"
)]
pub async fn delete_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    // 1. Verify realm access
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    // 2. Verify existence and ownership
    let existing = state
        .service
        .get_federation_provider(id)
        .await
        .map_err(ApiError::from)?;

    if existing.realm_id != Into::<Uuid>::into(realm.id) {
        return Err(ApiError::NotFound(
            "Provider not found in this realm".to_string(),
        ));
    }

    // 3. Delete
    state
        .service
        .delete_federation_provider(id)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
