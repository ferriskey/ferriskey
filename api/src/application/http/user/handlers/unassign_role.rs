use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::{entities::UnassignRoleInput, ports::UserService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UnassignRoleResponse {
    pub message: String,
    pub realm_name: String,
    pub user_id: Uuid,
}

#[utoipa::path(
    delete,
    path = "/{user_id}/roles/{role_id}",
    tag = "user",
    summary = "Unassign a role from a user in a realm",
    description = "Unassigns a specific role from a user in a realm. This action is irreversible and will remove the user's access to the role's permissions.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, body = UnassignRoleResponse, description = "Role unassigned successfully"),
        (status = 403, description = "Forbidden - You do not have permission to unassign roles"),
        (status = 404, description = "User or role not found")
    )
)]
pub async fn unassign_role(
    Path((realm_name, user_id, role_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UnassignRoleResponse>, ApiError> {
    state
        .service
        .unassign_role(
            identity,
            UnassignRoleInput {
                realm_name: realm_name.clone(),
                role_id,
                user_id,
            },
        )
        .await?;

    Ok(Response::OK(UnassignRoleResponse {
        message: format!("Role {role_id} unassigned from user {user_id} in realm {realm_name}"),
        realm_name,
        user_id,
    }))
}
