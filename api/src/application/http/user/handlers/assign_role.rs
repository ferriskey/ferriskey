use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::user::ports::UserService;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, user::entities::AssignRoleInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AssignRoleResponse {
    pub message: String,
    pub realm_name: String,
    pub user_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/{user_id}/roles/{role_id}",
    tag = "user",
    summary = "Assign a role to a user in a realm",
    description = "Assigns a specified role to a user within a given realm. This endpoint is used to manage user roles in the system.",
    responses(
        (status = 200, description = "Role assigned successfully", body = AssignRoleResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
)]
pub async fn assign_role(
    Path((realm_name, user_id, role_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AssignRoleResponse>, ApiError> {
    state
        .service
        .assign_role(
            identity,
            AssignRoleInput {
                realm_name: realm_name.clone(),
                user_id,
                role_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AssignRoleResponse {
        message: format!("Role {role_id} assigned to user {user_id} in realm {realm_name}"),
        realm_name,
        user_id,
    }))
}
