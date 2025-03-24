use uuid::Uuid;

use super::{
    entities::{error::SessionError, model::UserSession},
    ports::{UserSessionRepository, UserSessionService},
};

#[derive(Debug, Clone)]
pub struct UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    pub user_session_repository: U,
}

impl<U> UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    pub fn new(user_session_repository: U) -> Self {
        Self {
            user_session_repository,
        }
    }
}

impl<U> UserSessionService for UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    async fn create_session(
        &self,
        user_id: uuid::Uuid,
        realm_id: uuid::Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<UserSession, SessionError> {
        let session = UserSession::new(user_id, realm_id, user_agent, ip_address);

        self.user_session_repository.create(&session).await?;

        Ok(session)
    }
}
