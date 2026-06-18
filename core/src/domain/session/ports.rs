use chrono::Duration;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    session::entities::{SessionError, UserSession},
};

pub trait UserSessionService: Send + Sync {
    fn create_session(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_duration: Duration,
        soft_expiry_duration: Option<Duration>,
    ) -> impl Future<Output = Result<UserSession, SessionError>> + Send;
}

pub trait UserSessionManagementService: Send + Sync {
    fn list_sessions(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<UserSession>, CoreError>> + Send;

    fn revoke_session(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
        session_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait UserSessionRepository: Send + Sync {
    fn create(
        &self,
        session: &UserSession,
    ) -> impl Future<Output = Result<(), SessionError>> + Send;

    fn find_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> impl Future<Output = Result<UserSession, SessionError>> + Send;

    fn find_all_by_user_and_realm(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<UserSession>, SessionError>> + Send;

    fn find_by_id(
        &self,
        session_id: Uuid,
    ) -> impl Future<Output = Result<Option<UserSession>, SessionError>> + Send;

    fn delete(&self, id: &Uuid) -> impl Future<Output = Result<(), SessionError>> + Send;

    fn update_last_seen(
        &self,
        session_id: Uuid,
    ) -> impl Future<Output = Result<(), SessionError>> + Send;
}
