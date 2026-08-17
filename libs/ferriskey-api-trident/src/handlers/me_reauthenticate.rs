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
    trident::ports::{ReauthenticateInput, TridentService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use zeroize::Zeroize;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MeReauthenticateRequest {
    #[validate(length(min = 1, max = 1024))]
    pub password: String,
    /// Required when the user has an authenticator app configured.
    pub otp_code: Option<String>,
}

impl Zeroize for MeReauthenticateRequest {
    fn zeroize(&mut self) {
        self.password.zeroize();
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeReauthenticateResponse {
    pub verified: bool,
    /// Short-lived, single-use, user-bound step-up token. Must be presented on
    /// the sensitive self-service operations it unlocks (`/me/totp/verify`,
    /// `/me/passkey/registration`, `DELETE /me/credentials/{id}`).
    pub step_up_token: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/reauthenticate",
    tag = "auth",
    summary = "Re-authenticate the signed-in user",
    description = "Verifies the account password and, when an authenticator is configured, the current OTP code. Required before sensitive self-service operations such as re-setting up two-factor authentication.",
    request_body = MeReauthenticateRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Re-authentication succeeded", body = MeReauthenticateResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_reauthenticate(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(mut payload): ValidateJson<MeReauthenticateRequest>,
) -> Result<Response<MeReauthenticateResponse>, ApiError> {
    // Take the password out so we own it; the request struct's `Zeroize` impl
    // still scrubs the (now empty) field and the OTP code below.
    let password = std::mem::take(&mut payload.password);
    let otp_code = payload.otp_code.take();

    let result = state
        .service
        .reauthenticate(identity, ReauthenticateInput { password, otp_code })
        .await
        .map_err(ApiError::from)?;

    // Scrub any remaining sensitive material from the inbound request.
    payload.zeroize();

    Ok(Response::OK(MeReauthenticateResponse {
        verified: true,
        step_up_token: result.step_up_token,
    }))
}
