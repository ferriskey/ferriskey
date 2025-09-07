use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    user::validators::UpdateUserValidator,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::{entities::UpdateUserInput, ports::UserService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateUserResponse {
    pub data: User,
}

#[utoipa::path(
    put,
    path = "/{user_id}",
    tag = "user",
    summary = "Update a user in a realm",
    description = "Updates an existing user in a specific realm. The user must exist and the request must include the necessary fields to update.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body(
        content = UpdateUserValidator,
        description = "User to update",
        content_type = "application/json",
    ),
    responses(
        (status = 200, body = UpdateUserResponse, description = "User updated successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    Path(realm_name): Path<String>,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateUserValidator>,
) -> Result<Response<UpdateUserResponse>, ApiError> {
    let user = state
        .service
        .update_user(
            identity,
            UpdateUserInput {
                user_id,
                realm_name,
                firstname: payload.firstname,
                lastname: payload.lastname,
                email: payload.email,
                email_verified: payload.email_verified,
                enabled: payload.enabled.unwrap_or(true),
                required_actions: payload.required_actions,
            },
        )
        .await?;

    Ok(Response::OK(UpdateUserResponse { data: user }))
}
