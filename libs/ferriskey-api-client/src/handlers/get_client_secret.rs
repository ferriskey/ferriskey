use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{api_error::ApiError, response::Response};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::GetClientInput;
use ferriskey_core::domain::client::ports::ClientService;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientSecretResponse {
    pub client_secret: Option<String>,
}

#[utoipa::path(
    get,
    path = "/{client_id}/client-secret",
    tag = "client",
    summary = "Reveal a confidential client's secret",
    description = "Returns the plaintext secret of a confidential client. Requires ManageClients (or ManageRealm) — strictly more than the ViewClients needed to read the client itself — and records a `client_secret_viewed` security event.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    responses(
        (status = 200, body = ClientSecretResponse),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Client not found"),
    )
)]
pub async fn get_client_secret(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientSecretResponse>, ApiError> {
    let client_secret = state
        .service
        .reveal_client_secret(
            identity,
            GetClientInput {
                client_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(ClientSecretResponse { client_secret }))
}
