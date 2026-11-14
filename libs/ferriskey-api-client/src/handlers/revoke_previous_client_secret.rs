use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::GetClientInput;
use ferriskey_core::domain::client::ports::ClientService;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct RevokePreviousClientSecretResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{client_id}/client-secret/previous",
    tag = "client",
    summary = "Revoke a client's previous secret",
    description = "Stops the secret a client had before its last rotation from working, before its grace period ends. Requires ManageClients (or ManageRealm) and records a `client_secret_revoked` security event.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    responses(
        (status = 200, body = RevokePreviousClientSecretResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found, or it has no previous secret", body = ApiErrorResponse),
    )
)]
pub async fn revoke_previous_client_secret(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RevokePreviousClientSecretResponse>, ApiError> {
    state
        .service
        .revoke_previous_client_secret(
            identity,
            GetClientInput {
                client_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(RevokePreviousClientSecretResponse {
        message: "Previous client secret revoked".to_string(),
    }))
}
