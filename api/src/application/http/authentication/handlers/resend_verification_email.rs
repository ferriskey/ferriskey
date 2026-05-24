use axum::extract::{Path, State};
use ferriskey_core::domain::email_verification::ports::EmailVerificationService;
use serde::Serialize;
use utoipa::ToSchema;

use crate::application::{
    decoded_token::OptionalToken,
    http::server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub struct ResendVerificationEmailResponse {
    pub message: String,
}

/// POST /realms/{realm_name}/login-actions/resend-verification-email
#[utoipa::path(
    post,
    path = "/login-actions/resend-verification-email",
    tag = "auth",
    summary = "Resend verification email",
    description = "Resend the email verification link to the user's email address. Requires a valid Bearer token in the Authorization header.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Verification email sent", body = ResendVerificationEmailResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn resend_verification_email_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    OptionalToken(token): OptionalToken,
) -> Result<Response<ResendVerificationEmailResponse>, ApiError> {
    let token = token.ok_or_else(|| ApiError::Unauthorized("Missing or invalid token".into()))?;
    let user_id = token.claims.sub;

    // Use webapp_url (frontend URL) as the base for verification links
    let verification_base_url = state.args.webapp_url.trim_end_matches('/').to_string();

    state
        .service
        .email_verification_service
        .send_verification_email(user_id, realm_name, verification_base_url)
        .await?;

    Ok(Response::OK(ResendVerificationEmailResponse {
        message: "Verification email sent successfully".to_string(),
    }))
}
