use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::{
    application::http::server::app_state::AppState,
    application::http::{
        client::validators::CreateClientValidator,
        server::{
            api_entities::api_error::{ApiError, ValidateJson},
            api_entities::response::Response,
        },
    },
    domain::client::{entities::model::Client, ports::client_service::ClientService},
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients")]
pub struct CreateClientRoute {
    pub realm_name: String,
}

#[utoipa::path(
    post,
    path = "",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client",
    request_body = CreateClientValidator,
)]
pub async fn create_client(
    CreateClientRoute { realm_name }: CreateClientRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<CreateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    state
        .client_service
        .create_client(payload, realm_name)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}
