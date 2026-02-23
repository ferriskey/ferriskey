use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::GetClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{scope_id}",
    summary = "Get a client scope",
    description = "Retrieves a client scope from the specified realm by its ID.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, description = "Client scope retrieved successfully", body = ClientScope),
    )
)]
pub async fn get_client_scope(
    Path((realm_name, scope_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientScope>, ApiError> {
    let scope = state
        .service
        .get_client_scope(
            identity,
            GetClientScopeInput {
                realm_name,
                scope_id,
            },
        )
        .await?;

    Ok(Response::OK(scope))
}
