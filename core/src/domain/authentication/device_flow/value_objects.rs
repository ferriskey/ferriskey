//! DTOs for the device authorization grant use cases (RFC 8628).

/// Input for the device authorization endpoint (RFC 8628 §3.1).
pub struct InitiateDeviceFlowInput {
    pub realm_name: String,
    pub client_id: String,
    pub scope: Option<String>,
}

/// Output of the device authorization endpoint (RFC 8628 §3.2).
pub struct InitiateDeviceFlowOutput {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    /// Session lifetime, in seconds.
    pub expires_in: i64,
    /// Minimum polling interval, in seconds.
    pub interval: i64,
}

/// Input used when the end user submits a code on the verification page
/// (RFC 8628 §3.3).
pub struct VerifyUserCodeInput {
    pub realm_name: String,
    pub user_code: String,
}

/// Input for the token endpoint when polling with the device code
/// (RFC 8628 §3.4).
pub struct PollDeviceTokenInput {
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub device_code: String,
}
