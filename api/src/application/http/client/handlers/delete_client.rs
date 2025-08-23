use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::client::use_cases::delete_client_use_case::DeleteClientUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteClientResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{client_id}",
    summary = "Delete a client",
    description = "Deletes a client from the specified realm. This action is irreversible and will remove all associated data.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = DeleteClientResponse, description = "Client deleted successfully"),
    ),
)]
pub async fn delete_client(
    Path(realm_name): Path<String>,
    Path(client_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteClientResponse>, ApiError> {
    info!(
        "Deleting client with ID {} in realm {}",
        client_id, realm_name
    );
    state
        .use_case_bundle
        .delete_client_use_case
        .execute(
            identity,
            DeleteClientUseCaseParams {
                client_id,
                realm_name: realm_name.clone(),
            },
        )
        .await?;

    Ok(Response::OK(DeleteClientResponse {
        message: format!("Client with ID {client_id} in realm {realm_name} deleted successfully"),
    }))
}
