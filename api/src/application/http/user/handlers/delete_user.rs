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
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteUserResponse {
    pub count: u32,
}

#[utoipa::path(
    delete,
    path = "/{user_id}",
    tag = "user",
    summary = "Delete a user in a realm",
    description = "Deletes a user in a realm. This action is irreversible and will remove all associated data.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = DeleteUserResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_user(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteUserResponse>, ApiError> {
    let count = state
        .service
        .delete_user(identity, realm_name, user_id)
        .await?;

    Ok(Response::OK(DeleteUserResponse {
        count: count as u32,
    }))
}
