use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::api_error::{ApiError, ApiErrorResponse},
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UnlockUserResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/{user_id}/unlock",
    tag = "user",
    summary = "Unlock a locked user account",
    description = "Clears the lockout state (failed_login_attempts and locked_until) for the specified user. Requires manage-users permission.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "User account unlocked successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn unlock_user(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .unlock_user(identity, realm_name, user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
