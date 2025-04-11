use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::authentication::{
    entities::auth_session::{AuthSession, AuthSessionError},
    ports::auth_session::AuthSessionRepository,
};

#[derive(Clone)]
pub struct PostgresAuthSessionRepository {
    pub pool: PgPool,
}

impl PostgresAuthSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AuthSessionRepository for PostgresAuthSessionRepository {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthSessionError> {
        sqlx::query!(
            "INSERT INTO auth_sessions (id, realm_id, client_id, redirect_uri, response_type, scope, state, nonce, user_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            session.id,
            session.realm_id,
            session.client_id,
            session.redirect_uri,
            session.response_type,
            session.scope,
            session.state,
            session.nonce,
            session.user_id,
            session.created_at,
            session.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Error creating session: {:?}", e);
            AuthSessionError::CreateSessionError
        })?;

        Ok(session.clone())
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthSessionError> {
        let session = sqlx::query_as!(
            AuthSession,
            "SELECT * FROM auth_sessions WHERE id = $1 LIMIT 1",
            session_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Error getting session: {:?}", e);
            AuthSessionError::NotFound
        })?;

        Ok(session)
    }
}
