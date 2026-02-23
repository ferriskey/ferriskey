use crate::application::http::{
    client::validators::UpdateClientValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
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
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateClientResponse {
    pub data: Client,
}

#[utoipa::path(
    patch,
    path = "/{client_id}",
    summary = "Update a client",
    description = "Updates an existing client in the specified realm. This endpoint allows you to modify client details such as name, client ID, and enabled status.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = UpdateClientValidator,
    responses(
        (status = 200, description = "Client updated successfully", body = UpdateClientResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_client(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateClientValidator>,
) -> Result<Response<UpdateClientResponse>, ApiError> {
    let client = state
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
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateClientResponse { data: client }))
}
