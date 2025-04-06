use std::sync::Arc;

use async_trait::async_trait;
use tracing::error;

use crate::{
    domain::authentication::{
        entities::auth_session::{AuthSession, AuthSessionError},
        ports::auth_session::AuthSessionRepository,
    },
    infrastructure::db::postgres::Postgres,
};

pub struct PostgresAuthSessionRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresAuthSessionRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

#[async_trait]
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
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|e| {
            error!("Error creating session: {:?}", e);
            AuthSessionError::CreateSessionError
        })?;

        Ok(session.clone())
    }
}
