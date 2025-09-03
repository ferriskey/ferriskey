use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::application::trident::use_cases::challenge_otp_use_case::ChallengeOtpUseCaseInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
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
    responses(
        (status = 200, body = ChallengeOtpResponse)
    )
)]
pub async fn challenge_otp(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ChallengeOtpRequest>,
) -> Result<Response<ChallengeOtpResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let result = state
        .use_case_bundle
        .challenge_otp_use_case
        .execute(ChallengeOtpUseCaseInput {
            code: payload.code,
            identity,
            session_code,
        })
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let response = ChallengeOtpResponse {
        url: result.login_url,
    };

    Ok(Response::OK(response))
}
