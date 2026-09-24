use ferriskey_core::domain::trident::ports::WebAuthnRpInfo;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RequestElevationValidator {
    #[serde(default)]
    pub password: Option<String>,

    #[serde(default)]
    pub otp_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChangeOwnPasswordValidator {
    pub elevation_id: Uuid,

    #[validate(length(min = 1, message = "current_password is required"))]
    #[serde(default)]
    pub current_password: String,

    #[validate(length(min = 1, message = "new_password is required"))]
    #[serde(default)]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct StartOwnOtpEnrollmentValidator {
    pub elevation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ConfirmOwnOtpEnrollmentValidator {
    pub elevation_id: Uuid,

    #[validate(length(min = 1, message = "code is required"))]
    #[serde(default)]
    pub code: String,

    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ElevationOnlyValidator {
    pub elevation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ConfirmOwnPasskeyRegistrationValidator {
    pub elevation_id: Uuid,

    #[schema(value_type = Object)]
    pub credential: RegisterPublicKeyCredential,
}

pub fn webauthn_rp_info_from_webapp_url(webapp_url: &str) -> WebAuthnRpInfo {
    let rp_id = Url::parse(webapp_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "localhost".to_string());

    WebAuthnRpInfo {
        rp_id,
        allowed_origin: webapp_url.to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateOwnProfileValidator {
    #[serde(default)]
    #[validate(length(min = 1, message = "username cannot be empty"))]
    pub username: Option<String>,

    #[serde(default)]
    pub firstname: Option<String>,

    #[serde(default)]
    pub lastname: Option<String>,

    #[serde(default)]
    #[validate(email(message = "email must be a valid email"))]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateOwnLocaleValidator {
    pub locale: Option<String>,
}
