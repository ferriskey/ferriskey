use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::DeleteClientScopeInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteClientScopeResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{scope_id}",
    summary = "Delete a client scope",
    description = "Deletes a client scope from the specified realm. This action is irreversible.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, body = DeleteClientScopeResponse, description = "Client scope deleted successfully"),
    ),
)]
pub async fn delete_client_scope(
    Path((realm_name, scope_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteClientScopeResponse>, ApiError> {
    state
        .service
        .delete_client_scope(
            identity,
            DeleteClientScopeInput {
                realm_name: realm_name.clone(),
                scope_id,
            },
        )
        .await?;

    Ok(Response::OK(DeleteClientScopeResponse {
        message: format!(
            "Client scope with ID {scope_id} in realm {realm_name} deleted successfully"
        ),
    }))
}
