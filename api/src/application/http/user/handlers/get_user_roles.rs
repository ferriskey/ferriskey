use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::user::use_cases::get_user_roles_use_case::GetUserRolesUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::role::entities::Role;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetUserRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    summary = "Get all roles for a specific user",
    path = "/{user_id}/roles",
    tag = "user",
    description = "Retrieves all roles associated with a user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = GetUserRolesResponse),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_roles(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetUserRolesResponse>, ApiError> {
    let roles = state
        .use_case_bundle
        .get_user_roles_use_case
        .execute(
            identity,
            GetUserRolesUseCaseParams {
                realm_name,
                user_id,
            },
        )
        .await?;
    Ok(Response::OK(GetUserRolesResponse { data: roles }))
}
