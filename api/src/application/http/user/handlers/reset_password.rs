use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::api_success::ApiSuccess;
use crate::application::http::server::app_state::AppState;
use crate::application::http::user::validators::ResetPasswordValidator;
use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::reset_password_use_case::ResetPasswordUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize, TypedPath)]
#[typed_path("/realms/{realm_name}/users/{user_id}/reset-password")]
pub struct ResetPasswordRoute {
    pub realm_name: String,
    pub user_id: Uuid,
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
        (status = 200, body = String, description = "Password reset successfully"),
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
) -> Result<ApiSuccess<String>, ApiError> {
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
                realm_name,
                user_id,
                value: payload.value,
            },
        )
        .await
        .map_err(|_| ApiError::InternalServerError("Internal server error".to_string()))?;

    Ok(ApiSuccess::new(
        StatusCode::OK,
        "Password reset successfully".to_string(),
    ))
}
