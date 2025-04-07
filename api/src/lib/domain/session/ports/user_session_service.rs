use crate::domain::session::entities::{error::SessionError, model::UserSession};
use uuid::Uuid;

pub trait UserSessionService: Clone + Send + Sync + 'static {
    fn create_session(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> impl Future<Output = Result<UserSession, SessionError>> + Send;
}
