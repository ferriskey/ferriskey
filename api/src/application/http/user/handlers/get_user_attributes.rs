use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::{GetUserAttributesInput, UserAttribute};
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
pub struct UserAttributesResponse {
    pub data: Vec<UserAttribute>,
}

#[utoipa::path(
    get,
    path = "/{user_id}/attributes",
    tag = "user",
    summary = "Get custom attributes for a user",
    description = "Returns all custom key-value attributes attached to the user.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Attributes retrieved successfully", body = UserAttributesResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_user_attributes(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserAttributesResponse>, ApiError> {
    let attributes = state
        .service
        .get_user_attributes(
            identity,
            GetUserAttributesInput {
                realm_name,
                user_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UserAttributesResponse { data: attributes }))
}
