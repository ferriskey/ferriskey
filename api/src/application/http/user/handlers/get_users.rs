use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::user::use_cases::get_users_use_case::GetUsersUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UsersResponse {
    pub data: Vec<User>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "user",
    summary = "Get all users in a realm",
    description = "Retrieves all users associated with a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Users retrieved successfully", body = UsersResponse),
    )
)]
pub async fn get_users(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UsersResponse>, ApiError> {
    let users = state
        .use_case_bundle
        .get_users_use_case
        .execute(identity, GetUsersUseCaseParams { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UsersResponse { data: users }))
}
