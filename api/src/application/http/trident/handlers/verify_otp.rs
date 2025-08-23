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
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::credential::ports::CredentialService;
use ferriskey_core::domain::trident::entities::TotpSecret;
use ferriskey_core::domain::trident::ports::TotpService;
use ferriskey_core::domain::user::entities::RequiredAction;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use tracing::error;
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
    let decoded = base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &payload.secret,
    )
    .ok_or_else(|| ApiError::BadRequest("Invalid OTP secret".to_string()))?;

    if decoded.len() != 20 {
        return Err(ApiError::BadRequest("Secret must be 160 bits".to_string()));
    }

    let user = match identity {
        Identity::User(user) => user,
        _ => return Err(ApiError::Forbidden("Only users can verify OTP".to_string())),
    };

    let secret = TotpSecret::from_base32(&payload.secret);

    let is_valid = state
        .service_bundle
        .totp_service
        .verify(&secret, &payload.code)
        .map_err(|_| ApiError::InternalServerError("Failed to verify OTP".to_string()))?;

    if !is_valid {
        error!("Invalid OTP code");
        return Err(ApiError::Unauthorized("Invalid OTP code".to_string()));
    }

    let credential_data = serde_json::json!({
      "subType": "totp",
      "digits": 6,
      "counter": 0,
      "period": 30,
      "algorithm": "HmacSha256",
    });

    state
        .service_bundle
        .credential_service
        .create_custom_credential(
            user.id,
            "otp".to_string(),
            secret.base32_encoded().to_string(),
            Some(payload.label),
            credential_data,
        )
        .await?;

    state
        .service_bundle
        .user_service
        .remove_required_action(user.id, RequiredAction::ConfigureOtp)
        .await?;

    Ok(Response::OK(VerifyOtpResponse {
        message: "OTP verified successfully".to_string(),
    }))
}
