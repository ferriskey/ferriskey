use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::ports::ScopeMappingService;
use ferriskey_core::domain::aegis::value_objects::UnassignClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/clients/{client_id}/optional-client-scopes/{scope_id}",
    summary = "Remove an optional client scope from a client",
    description = "Removes a client scope from the optional scopes of the specified client.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, description = "Optional scope removed successfully"),
    ),
)]
pub async fn unassign_optional_scope(
    Path((realm_name, client_id, scope_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .unassign_scope_from_client(
            identity,
            UnassignClientScopeInput {
                realm_name,
                client_id,
                scope_id,
            },
        )
        .await?;

    Ok(Response::OK(()))
}
