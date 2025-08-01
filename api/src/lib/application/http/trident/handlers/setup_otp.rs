use crate::application::{
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    url::FullUrl,
};
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::TotpService;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, Eq, ToSchema)]
#[typeshare]
pub struct SetupOtpResponse {
    pub secret: String,
    pub otpauth_url: String,
    pub issuer: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/setup-otp")]
pub struct SetupOtpRoute {
    pub realm_name: String,
}

#[utoipa::path(get, path = "/login-actions/setup-otp", tag = "auth")]
pub async fn setup_otp(
    SetupOtpRoute { realm_name }: SetupOtpRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response<SetupOtpResponse>, ApiError> {
    let issuer = format!("{base_url}/realms/{realm_name}");
    let user = match identity {
        Identity::User(user) => user,
        _ => return Err(ApiError::Forbidden("Only users can set up OTP".to_string())),
    };
    let secret = state
        .service_bundle
        .totp_service
        .generate_secret()
        .map_err(|_| ApiError::InternalServerError("Failed to generate OTP secret".to_string()))?;

    let otpauth_url =
        state
            .service_bundle
            .totp_service
            .generate_otpauth_uri(&issuer, &user.email, &secret);

    let response = SetupOtpResponse {
        issuer: format!("{base_url}/realms/{realm_name}"),
        otpauth_url,
        secret: secret.base32_encoded().to_string(),
    };

    Ok(Response::OK(response))
}
