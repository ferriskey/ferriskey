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
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::update_user_use_case::UpdateUserUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}")]
pub struct UpdateUserRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

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
    UpdateUserRoute {
        realm_name,
        user_id,
    }: UpdateUserRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateUserValidator>,
) -> Result<Response<UpdateUserResponse>, ApiError> {
    let user = state
        .use_case_bundle
        .update_user_use_case
        .execute(
            identity,
            UpdateUserUseCaseParams {
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
