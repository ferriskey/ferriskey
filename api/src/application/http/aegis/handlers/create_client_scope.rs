use crate::application::http::{
    aegis::validators::CreateClientScopeValidator,
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
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::CreateClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    post,
    path = "",
    summary = "Create a new client scope",
    description = "Creates a new client scope within the specified realm.",
    responses(
        (status = 201, body = ClientScope, description = "Client scope created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client-scope",
    request_body = CreateClientScopeValidator,
)]
pub async fn create_client_scope(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateClientScopeValidator>,
) -> Result<Response<ClientScope>, ApiError> {
    let scope = state
        .service
        .create_client_scope(
            identity,
            CreateClientScopeInput {
                realm_name,
                name: payload.name,
                description: payload.description,
                protocol: payload.protocol,
                is_default: payload.is_default,
            },
        )
        .await?;

    Ok(Response::Created(scope))
}
