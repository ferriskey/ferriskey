use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::ports::RoleService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteRoleResponse {
    pub message: String,
    pub realm_name: String,
    pub role_id: Uuid,
}

#[utoipa::path(
    delete,
    summary = "Delete a role in a realm",
    path = "/{role_id}",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, body = DeleteRoleResponse, description = "Role deleted successfully"),
        (status = 404, description = "Role not found"),
        (status = 400, description = "Invalid request data")
    ),
)]
pub async fn delete_role(
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteRoleResponse>, ApiError> {
    state
        .service
        .delete_role(identity, realm_name.clone(), role_id)
        .await?;

    Ok(Response::OK(DeleteRoleResponse {
        message: format!("Role with ID {role_id} in realm {realm_name} deleted successfully"),
        realm_name,
        role_id,
    }))
}
