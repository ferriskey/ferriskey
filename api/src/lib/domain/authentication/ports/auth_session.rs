use async_trait::async_trait;

use crate::domain::authentication::entities::auth_session::{AuthSession, AuthSessionError};

#[async_trait]
pub trait AuthSessionService: Send + Sync {}


#[async_trait]
pub trait AuthSessionRepository: Send + Sync {
    async fn create(
        &self,
        session: &AuthSession,
    ) -> Result<AuthSession, AuthSessionError>;
}