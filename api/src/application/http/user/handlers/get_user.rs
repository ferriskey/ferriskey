use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::ports::UserService;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, user::entities::GetUserInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
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
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserResponse>, ApiError> {
    let user = state
        .service
        .get_user(
            identity,
            GetUserInput {
                user_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UserResponse { data: user }))
}
