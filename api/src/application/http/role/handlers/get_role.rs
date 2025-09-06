use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::entities::Role;
use ferriskey_core::domain::role::ports::RoleService;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetRoleResponse {
    pub data: Role,
}

#[utoipa::path(
    get,
    summary = "Get a role by ID in a realm",
    path = "/{role_id}",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("role_id" = Uuid, Path, description = "Role ID")
    ),
    responses(
        (status = 200, body = GetRoleResponse),
        (status = 404, description = "Role not found")
    )
)]
pub async fn get_role(
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetRoleResponse>, ApiError> {
    info!(
        "Fetching role with ID: {} in realm: {}",
        role_id, realm_name
    );

    let role = state
        .service
        .get_role(identity, realm_name, role_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetRoleResponse { data: role }))
}
