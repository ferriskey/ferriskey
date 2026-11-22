use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::device_session::entities::{
    AccountEntry, DeviceSession, DeviceSessionError, SelectedAccount,
};
use crate::realm::scope::{RealmScope, Scoped, Unscoped};
use crate::session::entities::UserSession;
use crate::user::entities::User;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DeviceSessionRepository: Send + Sync {
    fn create(
        &self,
        device: &DeviceSession,
    ) -> impl Future<Output = Result<(), DeviceSessionError>> + Send;

    fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<Unscoped<DeviceSession>>, DeviceSessionError>> + Send;

    fn touch_last_seen(
        &self,
        device: &Scoped<DeviceSession>,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), DeviceSessionError>> + Send;

    fn delete_expired(
        &self,
        realm_id: Uuid,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, DeviceSessionError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SessionResumptionPort: Send + Sync {
    fn generate_secret(&self) -> String;

    fn hash_secret(&self, secret: &str) -> String;

    fn resumable_session(
        &self,
        scope: &RealmScope,
        session: Scoped<UserSession>,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> impl Future<Output = Result<(Scoped<UserSession>, User), CoreError>> + Send;
}

pub trait AccountChooserService: Send + Sync {
    fn attach_session_to_device(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        user_session_id: Uuid,
    ) -> impl Future<Output = Result<(DeviceSession, Option<String>), CoreError>> + Send;

    fn device_session_id(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
    ) -> impl Future<Output = Result<Option<Uuid>, CoreError>> + Send;

    fn list_entries(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        active_sso_token: Option<&str>,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<AccountEntry>, CoreError>> + Send;

    fn select_entry(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        session_id: Uuid,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> impl Future<Output = Result<SelectedAccount, CoreError>> + Send;

    fn detach_entry(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        session_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
