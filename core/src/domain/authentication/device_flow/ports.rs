use uuid::Uuid;

use crate::domain::authentication::device_flow::entities::{DeviceAuthSession, DeviceAuthStatus};
use crate::domain::authentication::device_flow::error::DeviceFlowError;
use crate::domain::authentication::device_flow::value_objects::{
    InitiateDeviceFlowOutput, InitiateDeviceFlowParams, PollDeviceTokenParams,
};
use crate::domain::authentication::entities::{AuthenticationError, JwtToken};
use crate::domain::authentication::value_objects::GenerateTokensForUserInput;
use crate::domain::common::entities::app_errors::CoreError;

/// Persistence contract for device authorization sessions (RFC 8628).
#[cfg_attr(test, mockall::automock)]
pub trait DeviceAuthRepository: Send + Sync {
    /// Persist a freshly created session.
    fn create(
        &self,
        session: &DeviceAuthSession,
    ) -> impl Future<Output = Result<DeviceAuthSession, AuthenticationError>> + Send;

    /// Look up a session by its opaque device code (used while polling).
    fn find_by_device_code(
        &self,
        device_code: Uuid,
    ) -> impl Future<Output = Result<Option<DeviceAuthSession>, AuthenticationError>> + Send;

    /// Look up a session by the human-readable user code (verification page).
    fn find_by_user_code(
        &self,
        user_code: String,
    ) -> impl Future<Output = Result<Option<DeviceAuthSession>, AuthenticationError>> + Send;

    /// Transition a session to a new status, optionally binding the approving
    /// user.
    fn update_status(
        &self,
        device_code: Uuid,
        status: DeviceAuthStatus,
        user_id: Option<Uuid>,
    ) -> impl Future<Output = Result<DeviceAuthSession, AuthenticationError>> + Send;

    /// Record that the device just polled, for `slow_down` enforcement.
    fn mark_polled(
        &self,
        device_code: Uuid,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;
}

/// Token issuance seam for the device flow.
///
/// Lets the device flow service mint tokens for an approved session without
/// depending on the full `AuthService` surface. Implemented by the
/// authentication service (delegating to its `generate_tokens_for_user` /
/// `exchange_token` issuance path).
#[cfg_attr(test, mockall::automock)]
pub trait DeviceTokenIssuer: Send + Sync {
    fn issue_tokens_for_user(
        &self,
        input: GenerateTokensForUserInput,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
}

/// Business logic for the OAuth 2.0 Device Authorization Grant (RFC 8628).
pub trait DeviceFlowService: Send + Sync {
    /// Device authorization endpoint: create a session, generate a unique
    /// `user_code`, persist it, and fire `auth.device_flow.initiated`.
    fn initiate(
        &self,
        params: InitiateDeviceFlowParams,
    ) -> impl Future<Output = Result<InitiateDeviceFlowOutput, DeviceFlowError>> + Send;

    /// Verification page: bind the approving user and mark the session
    /// approved.
    fn verify_user_code(
        &self,
        user_code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), DeviceFlowError>> + Send;

    /// Verification page: mark the session denied and fire
    /// `auth.device_flow.denied`.
    fn deny(
        &self,
        user_code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), DeviceFlowError>> + Send;

    /// Token endpoint: advance the polling state machine, returning a
    /// [`JwtToken`] once the session is approved.
    fn poll(
        &self,
        params: PollDeviceTokenParams,
    ) -> impl Future<Output = Result<JwtToken, DeviceFlowError>> + Send;
}
