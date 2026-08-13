use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{SetupOtpInput, TridentService},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::validators::otp_issuer_from_webapp_url;

#[derive(Debug, Serialize, PartialEq, Eq, ToSchema)]
pub struct MeTotpSetupResponse {
    pub secret: String,
    pub otpauth_url: String,
    pub issuer: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/totp/setup",
    tag = "auth",
    summary = "Start TOTP setup for the signed-in user",
    description = "Generates a fresh TOTP secret and otpauth:// URI for the authenticated user. Bearer-only self-service endpoint; no session cookie required.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "TOTP setup payload generated", body = MeTotpSetupResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_totp_setup(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MeTotpSetupResponse>, ApiError> {
    let issuer = otp_issuer_from_webapp_url(&state.args.webapp_url, &realm_name);
    let result = state
        .service
        .setup_otp(identity, SetupOtpInput { issuer: issuer.clone() })
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string().into()))?;

    let response =
        MeTotpSetupResponse { issuer, otpauth_url: result.otpauth_uri, secret: result.secret };

    Ok(Response::OK(response))
}
