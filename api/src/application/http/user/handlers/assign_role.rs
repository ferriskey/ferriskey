use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::assign_role_use_case::AssignRoleUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AssignRoleResponse {
    pub message: String,
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/roles/{role_id}")]
pub struct AssignRoleRoute {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/{user_id}/roles/{role_id}",
    tag = "user",
    summary = "Assign a role to a user in a realm",
    description = "Assigns a specified role to a user within a given realm. This endpoint is used to manage user roles in the system.",
    responses(
        (status = 200, body = AssignRoleResponse, description = "Role assigned successfully"),
        (status = 404, description = "User or role not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
)]
pub async fn assign_role(
    AssignRoleRoute {
        realm_name,
        user_id,
        role_id,
    }: AssignRoleRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AssignRoleResponse>, ApiError> {
    state
        .use_case_bundle
        .assign_role_use_case
        .execute(
            identity,
            AssignRoleUseCaseParams {
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
