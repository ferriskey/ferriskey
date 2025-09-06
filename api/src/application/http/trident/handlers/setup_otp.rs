use crate::application::{
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    url::FullUrl,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{SetupOtpInput, TridentService},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, Eq, ToSchema)]
pub struct SetupOtpResponse {
    pub secret: String,
    pub otpauth_url: String,
    pub issuer: String,
}

#[utoipa::path(
    get,
    path = "/login-actions/setup-otp",
    tag = "auth",
    summary = "Setup OTP for user authentication",
    description = "Sets up a One-Time Password (OTP) for user authentication. This is typically used in multi-factor authentication scenarios.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = SetupOtpResponse, description = "OTP setup successful"),
        (status = 403, description = "Forbidden - Only users can set up OTP"),
        (status = 500, description = "Internal Server Error - Failed to generate OTP secret")
    )
)]
pub async fn setup_otp(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response<SetupOtpResponse>, ApiError> {
    let issuer = format!("{base_url}/realms/{realm_name}");
    let result = state
        .service
        .setup_otp(identity, SetupOtpInput { issuer })
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let response = SetupOtpResponse {
        issuer: format!("{base_url}/realms/{realm_name}"),
        otpauth_url: result.otpauth_uri,
        secret: result.secret,
    };

    Ok(Response::OK(response))
}
