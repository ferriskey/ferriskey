use crate::application::http::{
    role::validators::UpdateRolePermissionsValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::application::role::use_cases::update_role_permissions_use_case::UpdateRolePermissionsUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::entities::Role;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

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
    let role = state
        .use_case_bundle
        .update_role_permissions_use_case
        .execute(
            identity,
            UpdateRolePermissionsUseCaseParams {
                permissions: payload.permissions,
                role_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(UpdateRolePermissionsResponse { data: role }))
}
