use uuid::Uuid;

use crate::domain::session::entities::{SessionError, UserSession};

pub trait UserSessionService: Send + Sync {
    fn create_session(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> impl Future<Output = Result<UserSession, SessionError>> + Send;
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
    fn delete(&self, id: &Uuid) -> impl Future<Output = Result<(), SessionError>> + Send;
}
