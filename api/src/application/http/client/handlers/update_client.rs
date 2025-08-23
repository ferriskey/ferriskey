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
use ferriskey_core::application::client::use_cases::update_client_use_case::UpdateClientUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use ferriskey_core::domain::client::value_objects::UpdateClientRequest;
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
    Path(realm_name): Path<String>,
    Path(client_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    state
        .use_case_bundle
        .update_client_use_case
        .execute(
            identity,
            UpdateClientUseCaseParams {
                client_id,
                realm_name,
                payload: UpdateClientRequest {
                    name: payload.name,
                    client_id: payload.client_id,
                    enabled: payload.enabled,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}
