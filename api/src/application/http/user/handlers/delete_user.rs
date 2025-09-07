use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
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
    responses(
        (status = 200, body = DeleteUserResponse, description = "User deleted successfully"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Forbidden: User does not have permission to delete this user")
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
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
