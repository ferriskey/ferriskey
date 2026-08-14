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

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MeReauthenticateRequest {
    pub password: String,
    /// Required when the user has an authenticator app configured.
    pub otp_code: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeReauthenticateResponse {
    pub verified: bool,
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
    ValidateJson(payload): ValidateJson<MeReauthenticateRequest>,
) -> Result<Response<MeReauthenticateResponse>, ApiError> {
    state
        .service
        .reauthenticate(
            identity,
            ReauthenticateInput {
                password: payload.password,
                otp_code: payload.otp_code,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(MeReauthenticateResponse { verified: true }))
}
