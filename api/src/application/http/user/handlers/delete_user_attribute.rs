use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::DeleteUserAttributeInput;
use ferriskey_core::domain::user::ports::UserService;
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
pub struct DeleteUserAttributeResponse {}

#[utoipa::path(
    delete,
    path = "/{user_id}/attributes/{key}",
    tag = "user",
    summary = "Delete a custom attribute from a user",
    description = "Removes the attribute with the given key from the user.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("key" = String, Path, description = "Attribute key to delete"),
    ),
    responses(
        (status = 200, description = "Attribute deleted successfully", body = DeleteUserAttributeResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User or attribute not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn delete_user_attribute(
    Path((realm_name, user_id, key)): Path<(String, Uuid, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteUserAttributeResponse>, ApiError> {
    state
        .service
        .delete_user_attribute(
            identity,
            DeleteUserAttributeInput {
                realm_name,
                user_id,
                key,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteUserAttributeResponse {}))
}
