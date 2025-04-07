use crate::domain::session::entities::{error::SessionError, model::UserSession};
use uuid::Uuid;

pub trait UserSessionRepository: Clone + Send + Sync + 'static {
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
