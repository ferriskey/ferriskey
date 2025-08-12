use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::application::client::use_cases::get_clients_use_case::GetClientsUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients")]
pub struct GetClientsRoute {
    pub realm_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
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
    GetClientsRoute { realm_name }: GetClientsRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientsResponse>, ApiError> {
    let clients = state
        .use_case_bundle
        .get_clients_use_case
        .execute(identity, GetClientsUseCaseParams { realm_name })
        .await?;

    Ok(Response::OK(ClientsResponse { data: clients }))
}
