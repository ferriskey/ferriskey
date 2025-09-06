use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::{entities::DeleteClientInput, ports::ClientService};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteClientResponse {
    pub message: String,
    pub realm_name: String,
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
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteClientResponse>, ApiError> {
    info!(
        "Deleting client with ID {} in realm {}",
        client_id, realm_name
    );

    state
        .service
        .delete_client(
            identity,
            DeleteClientInput {
                client_id,
                realm_name: realm_name.clone(),
            },
        )
        .await?;

    Ok(Response::OK(DeleteClientResponse {
        message: format!("Client with ID {client_id} in realm {realm_name} deleted successfully"),
        realm_name,
    }))
}
