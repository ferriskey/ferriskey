use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::{
        auth::Identity,
        http::{
            server::{
                api_entities::{api_error::ApiError, response::Response},
                app_state::AppState,
            },
            user::policies::user_role_policies::UserRolePolicy,
        },
    },
    domain::{
        realm::ports::realm_service::RealmService, user::ports::user_role_service::UserRoleService,
    },
};

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
    let realm = state
        .realm_service
        .get_by_name(realm_name.clone())
        .await
        .map_err(ApiError::from)?;

    if !UserRolePolicy::delete(identity, state.clone(), realm).await? {
        return Err(ApiError::Forbidden(
            "You do not have permission to unassign roles".to_string(),
        ));
    }

    state
        .user_role_service
        .revoke_role(user_id, role_id)
        .await?;

    Ok(Response::OK(UnassignRoleResponse {
        message: format!("Role {role_id} unassigned from user {user_id} in realm {realm_name}"),
    }))
}
