use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::http::user::validators::ResetPasswordValidator;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::reset_password_use_case::ResetPasswordUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, TypedPath)]
#[typed_path("/realms/{realm_name}/users/{user_id}/reset-password")]
pub struct ResetPasswordRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ResetPasswordResponse {
    pub message: String,
    pub user_id: Uuid,
    pub realm_name: String,
}

#[utoipa::path(
    put,
    path = "/{user_id}/reset-password",
    tag = "user",
    summary = "Reset user password",
    description = "Resets the password for a user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body(
        content = ResetPasswordValidator,
        description = "New password for the user",
        content_type = "application/json",
    ),
    responses(
        (status = 200, body = ResetPasswordResponse, description = "Password reset successfully"),
    )
)]
pub async fn reset_password(
    ResetPasswordRoute {
        user_id,
        realm_name,
    }: ResetPasswordRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ResetPasswordValidator>,
) -> Result<Response<ResetPasswordResponse>, ApiError> {
    info!(
        "reset password for user {:} in realm {:}",
        user_id, realm_name
    );
    state
        .use_case_bundle
        .reset_password_use_case
        .execute(
            identity,
            ResetPasswordUseCaseParams {
                realm_name: realm_name.clone(),
                user_id,
                value: payload.value,
            },
        )
        .await
        .map_err(|_| ApiError::InternalServerError("Internal server error".to_string()))?;

    Ok(Response::OK(ResetPasswordResponse {
        message: "Password reset successfully".to_string(),
        user_id,
        realm_name,
    }))
}
