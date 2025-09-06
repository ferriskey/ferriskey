use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    trident::validators::OtpVerifyRequest,
};
use axum::{Extension, extract::State};
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
        (status = 200, body = VerifyOtpResponse)
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
                label: Some(payload.label),
                secret: payload.secret,
            },
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Response::OK(VerifyOtpResponse {
        message: result.message,
    }))
}
