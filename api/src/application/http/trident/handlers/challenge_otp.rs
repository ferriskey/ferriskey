use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::{ChallengeOtpInput, TridentService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChallengeOtpRequest {
    #[validate(length(min = 6, max = 6, message = "OTP code must be exactly 6 digits"))]
    #[serde(default)]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct ChallengeOtpResponse {
    pub url: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/challenge-otp",
    tag = "auth",
    summary = "Challenge OTP for user authentication",
    description = "Challenges the user to provide a One-Time Password (OTP) for authentication. This is typically used in multi-factor authentication scenarios.",
    request_body = ChallengeOtpRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Successfully challenged OTP", body = ChallengeOtpResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn challenge_otp(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ChallengeOtpRequest>,
) -> Result<Response<ChallengeOtpResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))? // Ou un type d'erreur 401/403
        .value()
        .to_string();

    let result = state
        .service
        .challenge_otp(
            identity,
            ChallengeOtpInput {
                code: payload.code,
                session_code,
            },
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let response = ChallengeOtpResponse {
        url: result.login_url,
    };

    Ok(Response::OK(response))
}
