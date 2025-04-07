use std::sync::Arc;

use uuid::Uuid;

use crate::{
    domain::session::{
        entities::{error::SessionError, model::UserSession},
        ports::user_session_repository::UserSessionRepository,
    },
    infrastructure::db::postgres::Postgres,
};

#[derive(Debug, Clone)]
pub struct PostgresUserSessionRepository {
    postgres: Arc<Postgres>,
}

impl PostgresUserSessionRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

impl UserSessionRepository for PostgresUserSessionRepository {
    async fn create(&self, session: &UserSession) -> Result<(), SessionError> {
        sqlx::query!(
            "INSERT INTO user_sessions (id, user_id, realm_id, user_agent, ip_address, created_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            session.id,
            session.user_id,
            session.realm_id,
            session.user_agent,
            session.ip_address,
            session.created_at,
            session.expires_at
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| SessionError::CreateError)?;

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<UserSession, SessionError> {
        let user_session = sqlx::query_as!(
            UserSession,
            "SELECT * FROM user_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map_err(|_| SessionError::NotFound)?;

        Ok(user_session)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), SessionError> {
        sqlx::query!("DELETE FROM user_sessions WHERE id = $1", id)
            .execute(&*self.postgres.get_pool())
            .await
            .map_err(|_| SessionError::DeleteError)?;

        Ok(())
    }
}
