use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

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
#[typed_path("/realms/{realm_name}/clients/{client_id}")]
pub struct GetClientRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct GetClientResponse {
    pub data: Client,
}

#[utoipa::path(
    get,
    path = "/{client_id}",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Client retrieved successfully", body = GetClientResponse),
    )
)]
pub async fn get_client(
    GetClientRoute {
        realm_name,
        client_id,
    }: GetClientRoute,
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Response<GetClientResponse>, ApiError> {
    let _realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let client = state
        .client_service
        .get_by_id(client_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetClientResponse { data: client }))
}
