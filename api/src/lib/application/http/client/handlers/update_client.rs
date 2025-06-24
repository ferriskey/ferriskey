use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::{
        client::validators::{CreateClientValidator, UpdateClientValidator},
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
    },
    domain::client::{
        entities::{dto::UpdateClientDto, model::Client},
        ports::client_service::ClientService,
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}")]
pub struct UpdateClientRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[utoipa::path(
    patch,
    path = "/{client_id}",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = UpdateClientValidator,
)]
pub async fn update_client(
    UpdateClientRoute {
        realm_name,
        client_id,
    }: UpdateClientRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<UpdateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    state
        .client_service
        .update_client(
            client_id,
            realm_name,
            UpdateClientDto {
                name: payload.name,
                client_id: payload.client_id,
                enabled: payload.enabled,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}
