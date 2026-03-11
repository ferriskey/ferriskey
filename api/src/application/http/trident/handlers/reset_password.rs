use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::{
    authentication::{
        entities::JwtToken, ports::AuthService, value_objects::GenerateTokensForUserInput,
    },
    trident::ports::{CompletePasswordResetInput, TridentService, VerifyResetTokenInput},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::{
    http::server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    url::FullUrl,
};

const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    pub token_id: Uuid,
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyResetTokenRequest {
    pub token_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyResetTokenResponse {
    pub valid: bool,
}

#[utoipa::path(
    post,
    path = "/login-actions/verify-reset-token",
    tag = "auth",
    summary = "Verify a password reset token",
    description = "Checks if a password reset token exists and has not expired, without consuming it.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = VerifyResetTokenRequest,
    responses(
        (status = 200, description = "Token is valid", body = VerifyResetTokenResponse),
        (status = 401, description = "Token is invalid or expired", body = ApiErrorResponse),
        (status = 404, description = "Token not found", body = ApiErrorResponse),
    )
)]
pub async fn verify_reset_token(
    Path(_realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<VerifyResetTokenRequest>,
) -> Result<Response<VerifyResetTokenResponse>, ApiError> {
    state
        .service
        .verify_reset_token(VerifyResetTokenInput {
            token_id: payload.token_id,
        })
        .await?;

    Ok(Response::OK(VerifyResetTokenResponse { valid: true }))
}

#[utoipa::path(
    post,
    path = "/login-actions/reset-password",
    tag = "auth",
    summary = "Reset password with token",
    description = "Completes the password reset flow by verifying the token and setting a new password. Returns authentication tokens to log the user in directly.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully, returns auth tokens", body = JwtToken),
        (status = 400, description = "Invalid or expired token", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn reset_password_with_token(
    Path(_realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<ResetPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .service
        .complete_password_reset(CompletePasswordResetInput {
            token_id: payload.token_id,
            token: payload.token,
            new_password: payload.new_password,
        })
        .await?;

    let is_secure = base_url.starts_with("https://");

    let token = state
        .service
        .generate_tokens_for_user(GenerateTokensForUserInput {
            user_id: result.user_id,
            realm_id: result.realm_id,
            base_url,
        })
        .await?;

    let mut identity_cookie = Cookie::build((IDENTITY_COOKIE, token.access_token().to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        identity_cookie = identity_cookie.secure(true);
    }

    let cookie_value = HeaderValue::from_str(&identity_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        axum::Json(token),
    ))
}
