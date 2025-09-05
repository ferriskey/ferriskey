use crate::application::http::{
    client::validators::UpdateClientValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::client::entities::Client;
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::client::value_objects::UpdateClientRequest;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, client::entities::UpdateClientInput,
};
use uuid::Uuid;

#[utoipa::path(
    patch,
    path = "/{client_id}",
    summary = "Update a client",
    description = "Updates an existing client in the specified realm. This endpoint allows you to modify client details such as name, client ID, and enabled status.",
    responses(
        (status = 200, description = "Client updated successfully", body = Client),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = UpdateClientValidator,
)]
pub async fn update_client(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    state
        .service
        .update_client(
            identity,
            UpdateClientInput {
                client_id,
                realm_name,
                payload: UpdateClientRequest {
                    name: payload.name,
                    client_id: payload.client_id,
                    enabled: payload.enabled,
                    direct_access_grants_enabled: payload.direct_access_grants_enabled,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}
