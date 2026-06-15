//! DTOs for the device authorization grant use cases (RFC 8628).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;

/// Domain command for [`DeviceFlowService::initiate`]. The realm and client are
/// already resolved by the application layer; the domain only deals with ids.
pub struct InitiateDeviceFlowParams {
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub scope: Option<String>,
    /// Whether the client is allowed to use the device authorization grant.
    /// When `false`, the service short-circuits with `unauthorized_client`
    /// (RFC 6749 §5.2) before any session is created.
    pub oauth_device_code_grant_enabled: bool,
    /// Absolute verification URI the user visits to enter the code, e.g.
    /// `https://auth.example.com/realms/master/device`.
    pub verification_uri: String,
}

/// Domain command for [`DeviceFlowService::poll`].
pub struct PollDeviceTokenParams {
    pub device_code: Uuid,
    pub client_id: Uuid,
    /// Issuer base URL used to mint tokens once the session is approved.
    pub base_url: String,
}

/// Input for the device authorization endpoint (RFC 8628 §3.1).
pub struct InitiateDeviceFlowInput {
    pub realm_name: String,
    pub client_id: String,
    pub scope: Option<String>,
}

/// Output of the device authorization endpoint (RFC 8628 §3.2).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitiateDeviceFlowOutput {
    pub device_code: String,
    #[schema(example = "WDJB-MJHT")]
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    /// Session lifetime, in seconds.
    #[schema(example = 600)]
    pub expires_in: i64,
    /// Minimum polling interval, in seconds.
    #[schema(example = 5)]
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
