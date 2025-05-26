use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::{
        auth::Identity,
        http::server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
    domain::{
        client::{entities::model::Client, ports::client_service::ClientService},
        realm::ports::realm_service::RealmService,
    },
};

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

pub async fn get_clients(
    GetClientsRoute { realm_name }: GetClientsRoute,
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Response<ClientsResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let clients = state
        .client_service
        .get_by_realm_id(realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ClientsResponse { data: clients }))
}
