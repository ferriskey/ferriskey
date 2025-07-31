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
use uuid::Uuid;

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
        .service_bundle
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let client = state
        .service_bundle
        .client_service
        .get_by_id(client_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetClientResponse { data: client }))
}
