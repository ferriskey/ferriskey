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

use crate::validators::OtpVerifyRequest;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MeTotpVerifyResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/totp/verify",
    tag = "auth",
    summary = "Verify and save a TOTP credential for the signed-in user",
    description = "Confirms the code produced by the secret from /me/totp/setup and stores the TOTP credential. Bearer-only self-service endpoint; no session cookie required.",
    request_body = OtpVerifyRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "TOTP credential saved", body = MeTotpVerifyResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_totp_verify(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<OtpVerifyRequest>,
) -> Result<Response<MeTotpVerifyResponse>, ApiError> {
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
        .map_err(ApiError::from)?;

    Ok(Response::OK(MeTotpVerifyResponse {
        message: result.message,
    }))
}
