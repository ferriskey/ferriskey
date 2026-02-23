use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    role::entities::permission::Permissions,
    user::{entities::GetUserPermissionsInput, ports::UserService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserPermissionsResponse {
    pub data: Vec<Permissions>,
}

#[utoipa::path(
    get,
    path = "/{user_id}/permissions",
    tag = "user",
    summary = "Get user permissions by ID in a realm",
    description = "Retrieves the permissions assigned to a user by their ID in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User permissions retrieved successfully", body = UserPermissionsResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_user_permissions(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserPermissionsResponse>, ApiError> {
    let permissions = state
        .service
        .get_user_permissions(
            identity,
            GetUserPermissionsInput {
                realm_name,
                user_id,
            },
        )
        .await?;

    Ok(Response::OK(UserPermissionsResponse { data: permissions }))
}
