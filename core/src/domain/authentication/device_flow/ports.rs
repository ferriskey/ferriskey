use uuid::Uuid;

use crate::domain::authentication::device_flow::entities::{DeviceAuthSession, DeviceAuthStatus};
use crate::domain::authentication::entities::AuthenticationError;

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
