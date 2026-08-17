use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use ferriskey_api_core::{
    api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson},
    app_state::AppState,
    url::FullUrl,
};
use ferriskey_core::domain::trident::ports::{
    CompletePasswordResetWithRecoveryCodeInput, TridentService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordWithRecoveryCodeRequest {
    #[validate(email)]
    pub email: String,
    pub recovery_code: String,
    pub recovery_code_format: String,
    pub new_password: String,
}

/// Response returned after a recovery code is successfully burned. A recovery
/// code is a *second* factor, so we do not mint a session here: a password-reset
/// link is emailed to the account and the new password is only applied when that
/// link is completed (standard email-reset flow).
#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordWithRecoveryCodeResponse {
    /// Human-readable status. The actual reset link is delivered by email.
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/login-actions/reset-password-with-recovery-code",
    tag = "auth",
    summary = "Reset password with a recovery code",
    description = "Burns a one-time recovery code to unlock a password reset. The code is a second factor: a reset link is emailed to the account and the new password is applied only when that link is completed, so a leaked code cannot take over the account or bypass MFA on its own.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ResetPasswordWithRecoveryCodeRequest,
    responses(
        (status = 200, description = "Recovery code accepted, password reset email sent", body = ResetPasswordWithRecoveryCodeResponse),
        (status = 400, description = "Invalid or expired recovery code", body = ApiErrorResponse),
        (status = 404, description = "No account found for the email", body = ApiErrorResponse),
        (status = 423, description = "Account is temporarily locked due to too many failed attempts", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn reset_password_with_recovery_code(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<ResetPasswordWithRecoveryCodeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .complete_password_reset_with_recovery_code(CompletePasswordResetWithRecoveryCodeInput {
            realm_name,
            email: payload.email,
            code: payload.recovery_code,
            format: payload.recovery_code_format,
            new_password: payload.new_password,
            base_url,
        })
        .await
        .map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::OK,
        axum::Json(ResetPasswordWithRecoveryCodeResponse {
            message: "Recovery code accepted. A password reset link has been sent to your email."
                .to_string(),
        }),
    ))
}
