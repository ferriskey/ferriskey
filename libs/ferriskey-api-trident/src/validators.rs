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

/// Derives a short TOTP issuer label (eTLD+1) from the webapp URL, e.g.
/// `https://account.example.com` -> `example.com`. Password managers group
/// authenticator entries by this label, so keeping it short and stable
/// avoids the long API hostname showing up in the OTP record.
///
/// Falls back to the realm name when no usable host can be parsed.
pub fn otp_issuer_from_webapp_url(webapp_url: &str, realm_name: &str) -> String {
    let host = Url::parse(webapp_url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_default();

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        parts[parts.len() - 2..].join(".")
    } else if !host.is_empty() {
        host
    } else {
        realm_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_issuer_uses_last_two_host_labels() {
        assert_eq!(
            otp_issuer_from_webapp_url("https://account.example.com", "demo"),
            "example.com"
        );
        assert_eq!(
            otp_issuer_from_webapp_url("https://api.sub.example.co.uk", "demo"),
            "co.uk"
        );
    }

    #[test]
    fn otp_issuer_keeps_two_part_hosts() {
        assert_eq!(otp_issuer_from_webapp_url("https://example.com", "demo"), "example.com");
        assert_eq!(otp_issuer_from_webapp_url("http://localhost:3000", "demo"), "localhost");
    }

    #[test]
    fn otp_issuer_falls_back_to_realm_name() {
        assert_eq!(otp_issuer_from_webapp_url("not a url", "demo"), "demo");
        assert_eq!(otp_issuer_from_webapp_url("", "demo"), "demo");
    }
}
