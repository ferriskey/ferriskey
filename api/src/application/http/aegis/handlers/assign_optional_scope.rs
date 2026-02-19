use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::entities::ClientScopeMapping;
use ferriskey_core::domain::aegis::ports::ScopeMappingService;
use ferriskey_core::domain::aegis::value_objects::AssignClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    put,
    path = "/clients/{client_id}/optional-client-scopes/{scope_id}",
    summary = "Assign an optional client scope to a client",
    description = "Assigns a client scope as an optional scope to the specified client. Optional scopes are included only when explicitly requested.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, body = ClientScopeMapping, description = "Optional scope assigned successfully"),
    ),
)]
pub async fn assign_optional_scope(
    Path((realm_name, client_id, scope_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientScopeMapping>, ApiError> {
    let mapping = state
        .service
        .assign_scope_to_client(
            identity,
            AssignClientScopeInput {
                realm_name,
                client_id,
                scope_id,
                is_default: false,
                is_optional: true,
            },
        )
        .await?;

    Ok(Response::OK(mapping))
}
