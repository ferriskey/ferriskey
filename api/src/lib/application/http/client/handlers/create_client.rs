use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::{
    application::http::{
        client::validators::CreateClientValidator,
        server::{
            errors::{ApiError, ValidateJson},
            handlers::Response,
        },
    },
    domain::client::{entities::model::Client, ports::ClientService},
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients")]
pub struct CreateClientRoute {
    pub realm_name: String,
}

#[utoipa::path(
    post,
    path = "",
    tag = "client",
    request_body = CreateClientValidator,
)]
pub async fn create_client<C: ClientService>(
    CreateClientRoute { realm_name }: CreateClientRoute,
    Extension(client_service): Extension<Arc<C>>,
    ValidateJson(payload): ValidateJson<CreateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    client_service
        .create_client(payload, realm_name)
        .await
        .map_err(ApiError::from)
        .map(|client| Response::Created(client))
}
