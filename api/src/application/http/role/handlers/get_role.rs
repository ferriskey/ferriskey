use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::role::use_cases::get_role_use_case::GetRoleUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::entities::Role;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/roles/{role_id}")]
pub struct GetRoleRoute {
    pub realm_name: String,
    pub role_id: Uuid,
}

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
    GetRoleRoute {
        realm_name,
        role_id,
    }: GetRoleRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetRoleResponse>, ApiError> {
    info!(
        "Fetching role with ID: {} in realm: {}",
        role_id, realm_name
    );

    let role = state
        .use_case_bundle
        .get_role_use_case
        .execute(
            identity,
            GetRoleUseCaseParams {
                role_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetRoleResponse { data: role }))
}
