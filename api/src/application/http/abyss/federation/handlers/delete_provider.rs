use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::ports::FederationService, authentication::value_objects::Identity,
};
use serde::Serialize;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Serialize, PartialEq, ToSchema)]
pub struct DeleteProviderResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/federation/providers/{id}",
    summary = "Delete a federation provider",
    responses(
        (status = 204, description = "Provider deleted", body = DeleteProviderResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden", body = ApiError),
        (status = 404, description = "Realm or Provider not found", body = ApiError),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "federation"
)]
pub async fn delete_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteProviderResponse>, ApiError> {
    info!(
        "Deleting federation provider with id {} in realm {}",
        id, realm_name
    );
    state
        .service
        .delete_federation_provider(identity, id, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteProviderResponse {
        message: "Provider deleted successfully".to_string(),
    }))
}
