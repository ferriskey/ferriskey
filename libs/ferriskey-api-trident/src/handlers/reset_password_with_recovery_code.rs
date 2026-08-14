use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_api_core::{
    api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson},
    app_state::AppState,
    url::FullUrl,
};
use ferriskey_core::domain::{
    authentication::{
        entities::JwtToken, ports::AuthService, value_objects::GenerateTokensForUserInput,
    },
    trident::ports::{CompletePasswordResetWithRecoveryCodeInput, TridentService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordWithRecoveryCodeRequest {
    #[validate(email)]
    pub email: String,
    pub recovery_code: String,
    pub recovery_code_format: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompletePasswordResetWithRecoveryCodeResponse {
    #[serde(flatten)]
    pub token: JwtToken,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/login-actions/reset-password-with-recovery-code",
    tag = "auth",
    summary = "Reset password with a recovery code",
    description = "Resets the password for the account matching the email using a one-time recovery code instead of the email reset token. Consumes the code and returns authentication tokens.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ResetPasswordWithRecoveryCodeRequest,
    responses(
        (status = 200, description = "Password reset successfully, returns auth tokens", body = CompletePasswordResetWithRecoveryCodeResponse),
        (status = 400, description = "Invalid or expired recovery code", body = ApiErrorResponse),
        (status = 404, description = "No account found for the email", body = ApiErrorResponse),
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
        .validate_password_policy(realm_name.clone(), &payload.new_password)
        .await?;

    let result = state
        .service
        .complete_password_reset_with_recovery_code(CompletePasswordResetWithRecoveryCodeInput {
            realm_name,
            email: payload.email,
            code: payload.recovery_code,
            format: payload.recovery_code_format,
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
            client_id: None,
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
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        axum::Json(CompletePasswordResetWithRecoveryCodeResponse { token }),
    ))
}
