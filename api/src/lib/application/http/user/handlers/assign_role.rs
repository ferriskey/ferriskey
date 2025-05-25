use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    domain::user::ports::user_role_service::UserRoleService,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssignRoleResponse {
    pub message: String,
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
    path = "/{realm_name}/users/{user_id}/roles/{role_id}",
    tag = "user",
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
) -> Result<Response<AssignRoleResponse>, ApiError> {
    state
        .user_role_service
        .assign_role(realm_name.clone(), user_id, role_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AssignRoleResponse {
        message: format!(
            "Role {} assigned to user {} in realm {}",
            role_id, user_id, realm_name
        ),
    }))
}
