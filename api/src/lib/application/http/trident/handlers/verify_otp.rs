use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::{
        auth::Identity,
        http::{
            server::{
                api_entities::{
                    api_error::{ApiError, ValidateJson},
                    response::Response,
                },
                app_state::AppState,
            },
            trident::validators::OtpVerifyRequest,
        },
    },
    domain::{
        credential::ports::credential_service::CredentialService,
        trident::{entities::TotpSecret, ports::TotpService},
        user::{entities::required_action::RequiredAction, ports::user_service::UserService},
    },
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[typeshare]
pub struct VerifyOtpResponse {
    pub message: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/verify-otp")]
pub struct VerifyOtpRoute {
    pub realm_name: String,
}

pub async fn verify_otp(
    VerifyOtpRoute { realm_name }: VerifyOtpRoute,
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

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&decoded);
    let secret = TotpSecret::from_bytes(bytes);

    let is_valid = state
        .totp_service
        .verify(&secret, &payload.code)
        .map_err(|_| ApiError::InternalServerError("Failed to verify OTP".to_string()))?;

    if !is_valid {
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
        .user_service
        .remove_required_action(user.id, RequiredAction::ConfigureOtp)
        .await?;

    Ok(Response::OK(VerifyOtpResponse {
        message: "OTP verified successfully".to_string(),
    }))
}
