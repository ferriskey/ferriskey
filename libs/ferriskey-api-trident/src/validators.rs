use ferriskey_core::domain::trident::ports::WebAuthnRpInfo;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;
use validator::Validate;

/// The TOTP secret is deliberately absent: it is read from the enrollment the server
/// recorded during `setup-otp`. Accepting it here let a caller enroll a secret of their
/// own choosing and verify a code against it (FK-003).
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct OtpVerifyRequest {
    pub code: String,
    pub label: String,
}

/// Derives the WebAuthn Relying Party info from the webapp URL.
///
/// The `rp_id` must be a valid domain that matches the origin,
/// not the server bind address (e.g. `localhost` not `0.0.0.0`).
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
