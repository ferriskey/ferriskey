use axum::extract::{Path, State};
use ferriskey_core::domain::trident::ports::{CompletePasswordResetInput, TridentService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    pub token_id: Uuid,
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/reset-password",
    tag = "auth",
    summary = "Reset password with token",
    description = "Completes the password reset flow by verifying the token and setting a new password.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResponse),
        (status = 400, description = "Invalid or expired token", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn reset_password_with_token(
    Path(_realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<ResetPasswordRequest>,
) -> Result<Response<ResetPasswordResponse>, ApiError> {
    state
        .service
        .complete_password_reset(CompletePasswordResetInput {
            token_id: payload.token_id,
            token: payload.token,
            new_password: payload.new_password,
        })
        .await?;

    Ok(Response::OK(ResetPasswordResponse {
        message: "Password has been reset successfully".to_string(),
    }))
}
