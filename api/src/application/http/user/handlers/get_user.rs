use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::get_user_use_case::GetUserUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}")]
pub struct GetUserRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UserResponse {
    pub data: User,
}

#[utoipa::path(
    get,
    path = "/{user_id}",
    tag = "user",
    summary = "Get a user by ID in a realm",
    description = "Retrieves a user by their ID in a specific realm. This endpoint returns detailed information about the user.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = UserResponse, description = "User retrieved successfully"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Forbidden: User does not have permission to access this user")
    )
)]
pub async fn get_user(
    GetUserRoute {
        realm_name,
        user_id,
    }: GetUserRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserResponse>, ApiError> {
    let user = state
        .use_case_bundle
        .get_user_use_case
        .execute(
            identity,
            GetUserUseCaseParams {
                user_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UserResponse { data: user }))
}
