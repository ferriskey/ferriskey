use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::{
    abyss::federation::ports::FederationService, authentication::value_objects::Identity,
};
use uuid::Uuid;

use crate::application::http::{
    server::api_entities::api_error::ApiError, server::app_state::AppState,
};

#[utoipa::path(
    delete,
    path = "/federation/providers/{id}",
    summary = "Delete a federation provider",
    responses(
        (status = 204, description = "Provider deleted"),
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
pub async fn delete_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .delete_federation_provider(identity, id, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
