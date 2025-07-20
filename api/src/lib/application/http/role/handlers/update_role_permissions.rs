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
            role::{policies::RolePolicy, validators::UpdateRolePermissionsValidator},
            server::{
                api_entities::{
                    api_error::{ApiError, ValidateJson},
                    response::Response,
                },
                app_state::AppState,
            },
        },
    },
    domain::{
        realm::ports::realm_service::RealmService,
        role::{
            entities::{UpdateRolePermissionsDto, models::Role},
            ports::RoleService,
        },
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/roles/{role_id}/permissions")]
pub struct UpdateRolePermissionsRoute {
    pub realm_name: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UpdateRolePermissionsResponse {
    pub data: Role,
}

#[utoipa::path(
  patch,
  summary = "Update a role in a realm",
  path = "/{role_id}/permissions",
  tag = "role",
  request_body = UpdateRolePermissionsValidator,
  params(
      ("realm_name" = String, Path, description = "Realm name"),
      ("role_id" = Uuid, Path, description = "Role ID"),
  ),
  responses(
      (status = 200, body = UpdateRolePermissionsResponse),
      (status = 404, description = "Role not found"),
      (status = 400, description = "Invalid request data")
  ),
)]
pub async fn update_role_permissions(
    UpdateRolePermissionsRoute {
        realm_name,
        role_id,
    }: UpdateRolePermissionsRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRolePermissionsValidator>,
) -> Result<Response<UpdateRolePermissionsResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    if !RolePolicy::update(identity, state.clone(), realm).await? {
        return Err(ApiError::Forbidden(
            "User not allowed to update role".to_string(),
        ));
    }
    let role = state
        .role_service
        .update_permissions_by_id(
            role_id,
            UpdateRolePermissionsDto {
                permissions: payload.permissions,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UpdateRolePermissionsResponse { data: role }))
}
