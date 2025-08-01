use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::unassign_role_use_case::UnassignRoleUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/roles/{role_id}")]
pub struct UnassignRoleRoute {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UnassignRoleResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{user_id}/roles/{role_id}",
    tag = "user",
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
    UnassignRoleRoute {
        realm_name,
        user_id,
        role_id,
    }: UnassignRoleRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UnassignRoleResponse>, ApiError> {
    state
        .use_case_bundle
        .unassign_role_use_case
        .execute(
            identity,
            UnassignRoleUseCaseParams {
                realm_name: realm_name.clone(),
                role_id,
                user_id,
            },
        )
        .await?;

    Ok(Response::OK(UnassignRoleResponse {
        message: format!("Role {role_id} unassigned from user {user_id} in realm {realm_name}"),
    }))
}
