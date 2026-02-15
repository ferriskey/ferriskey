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
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::entities::Role;
use ferriskey_core::domain::role::ports::RoleService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    summary = "Get all roles for a realm",
    path = "",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Roles retrieved successfully", body = GetRolesResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_roles(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetRolesResponse>, ApiError> {
    let roles = state
        .service
        .get_roles(identity, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetRolesResponse { data: roles }))
}
