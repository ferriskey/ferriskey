use crate::application::http::{
    client::validators::CreateClientValidator,
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
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use ferriskey_core::domain::client::{entities::CreateClientInput, ports::ClientService};

#[utoipa::path(
    post,
    path = "",
    summary = "Create a new client in a realm",
    description = "Creates a new client within the specified realm. This endpoint allows you to register a new client application that can interact with the realm's resources.",
    responses(
        (status = 201, body = Client, description = "Client created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client",
    request_body = CreateClientValidator,
)]
pub async fn create_client(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    let client = state
        .service
        .create_client(
            identity,
            CreateClientInput {
                client_id: payload.client_id,
                client_type: payload.client_type,
                public_client: payload.public_client,
                realm_name,
                enabled: payload.enabled,
                name: payload.name,
                protocol: payload.protocol,
                service_account_enabled: payload.service_account_enabled,
                direct_access_grants_enabled: payload.direct_access_grants_enabled,
            },
        )
        .await?;

    Ok(Response::Created(client))
}
