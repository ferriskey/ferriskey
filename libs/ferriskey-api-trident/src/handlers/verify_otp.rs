use crate::validators::OtpVerifyRequest;
use axum::{Extension, extract::State};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, VerifyOtpInput},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VerifyOtpResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/verify-otp",
    tag = "auth",
    summary = "Verify OTP for user authentication",
    description = "Verifies the One-Time Password (OTP) provided by the user. This is typically used in multi-factor authentication scenarios.",
    request_body = OtpVerifyRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "OTP verified successfully", body = VerifyOtpResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "No pending enrollment, or the code does not match it", body = ApiErrorResponse),
        (status = 403, description = "OTP is already configured and no re-configuration was requested", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn verify_otp(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<OtpVerifyRequest>,
) -> Result<Response<VerifyOtpResponse>, ApiError> {
    let result = state
        .service
        .verify_otp(
            identity,
            VerifyOtpInput {
                code: payload.code,
                // The login flow is already protected by a temporary login
                // token, so no step-up token is required here.
                step_up_token: None,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(VerifyOtpResponse {
        message: result.message,
    }))
}
