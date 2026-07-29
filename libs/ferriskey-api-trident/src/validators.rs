use ferriskey_core::domain::trident::ports::WebAuthnRpInfo;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct OtpVerifyRequest {
    pub code: String,
    pub label: String,
    pub secret: String,
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
