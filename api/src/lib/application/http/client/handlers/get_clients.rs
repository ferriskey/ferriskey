use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::realm::ports::RealmService;
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
    Extension(_identity): Extension<Identity>,
) -> Result<Response<ClientsResponse>, ApiError> {
    let realm = state
        .service_bundle
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let clients = state
        .service_bundle
        .client_service
        .get_by_realm_id(realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ClientsResponse { data: clients }))
}
