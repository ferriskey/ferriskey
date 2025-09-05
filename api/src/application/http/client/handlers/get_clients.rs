use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use ferriskey_core::domain::client::{entities::GetClientsInput, ports::ClientService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ClientsResponse {
    pub data: Vec<Client>,
}

#[utoipa::path(
    get,
    path = "",
    summary = "Get clients in a realm",
    description = "Retrieves all clients associated with a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Clients retrieved successfully", body = ClientsResponse),
    )
)]
pub async fn get_clients(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientsResponse>, ApiError> {
    let clients = state
        .service
        .get_clients(identity, GetClientsInput { realm_name })
        .await?;

    Ok(Response::OK(ClientsResponse { data: clients }))
}
