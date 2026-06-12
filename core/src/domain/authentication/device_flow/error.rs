use thiserror::Error;

use crate::domain::authentication::entities::AuthenticationError;
use crate::domain::common::entities::app_errors::CoreError;

/// Errors surfaced by the device authorization grant (RFC 8628).
///
/// The `Display` strings of the polling variants intentionally match the
/// `error` codes defined by RFC 8628 §3.5 so the HTTP layer can forward them
/// verbatim in the token endpoint response.
#[derive(Debug, Clone, Error)]
pub enum DeviceFlowError {
    /// The authorization request is still pending — the user has not yet
    /// approved or denied it (RFC 8628 §3.5 `authorization_pending`).
    #[error("authorization_pending")]
    AuthorizationPending,

    /// The device polled faster than the allowed interval (RFC 8628 §3.5
    /// `slow_down`).
    #[error("slow_down")]
    SlowDown,

    /// The session lifetime elapsed before approval (RFC 8628 §3.5
    /// `expired_token`).
    #[error("expired_token")]
    ExpiredToken,

    /// The end user denied the authorization request (RFC 8628 §3.5
    /// `access_denied`).
    #[error("access_denied")]
    AccessDenied,

    /// No session matches the supplied device code.
    #[error("invalid device code")]
    InvalidDeviceCode,

    /// No session matches the supplied user code.
    #[error("invalid user code")]
    InvalidUserCode,

    /// The polling client does not match the client that initiated the flow.
    #[error("invalid client")]
    InvalidClient,

    /// The client exists but is not allowed to use the device authorization
    /// grant — RFC 6749 §5.2 `unauthorized_client` ("not authorized to use
    /// this authorization grant type"). Surfaced from the device
    /// authorization endpoint when the operator has not enabled the grant
    /// for the client.
    #[error("unauthorized_client")]
    UnauthorizedClient,

    /// Could not generate a collision-free user code within the retry budget.
    #[error("failed to generate a unique user code")]
    UserCodeGenerationExhausted,

    /// Token issuance failed once the session was approved.
    #[error("token issuance failed: {0}")]
    TokenIssuance(String),

    /// An underlying repository error.
    #[error(transparent)]
    Repository(#[from] AuthenticationError),
}

impl From<DeviceFlowError> for CoreError {
    fn from(err: DeviceFlowError) -> Self {
        match err {
            DeviceFlowError::AuthorizationPending => CoreError::InvalidRequest,
            DeviceFlowError::SlowDown => CoreError::InvalidRequest,
            DeviceFlowError::ExpiredToken => CoreError::ExpiredToken,
            DeviceFlowError::AccessDenied => CoreError::Forbidden("access_denied".to_string()),
            DeviceFlowError::InvalidDeviceCode => CoreError::InvalidToken,
            DeviceFlowError::InvalidUserCode => CoreError::InvalidToken,
            DeviceFlowError::InvalidClient => CoreError::InvalidClient,
            DeviceFlowError::UnauthorizedClient => CoreError::InvalidClient,
            DeviceFlowError::UserCodeGenerationExhausted => CoreError::InternalServerError,
            DeviceFlowError::TokenIssuance(msg) => CoreError::TokenGenerationError(msg),
            DeviceFlowError::Repository(err) => err.into(),
        }
    }
}
