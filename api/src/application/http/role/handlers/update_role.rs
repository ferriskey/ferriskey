use crate::application::http::{
    role::validators::UpdateRoleValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::role::entities::Role;
use ferriskey_core::domain::role::ports::RoleService;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, role::entities::UpdateRoleInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateRoleResponse {
    pub data: Role,
}

#[utoipa::path(
  put,
  summary = "Update a role in a realm",
  path = "/{role_id}",
  tag = "role",
  request_body = UpdateRoleValidator,
  params(
      ("realm_name" = String, Path, description = "Realm name"),
      ("role_id" = Uuid, Path, description = "Role ID"),
  ),
  responses(
      (status = 200, body = UpdateRoleResponse),
      (status = 404, description = "Role not found"),
      (status = 400, description = "Invalid request data")
  ),
)]
pub async fn update_role(
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRoleValidator>,
) -> Result<Response<UpdateRoleResponse>, ApiError> {
    let role = state
        .service
        .update_role(
            identity,
            UpdateRoleInput {
                name: payload.name,
                description: payload.description,
                role_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(UpdateRoleResponse { data: role }))
}
