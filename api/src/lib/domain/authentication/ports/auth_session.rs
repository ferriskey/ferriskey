use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::authentication::entities::auth_session::{AuthSession, AuthSessionError};

#[async_trait]
pub trait AuthSessionService: Send + Sync + 'static {
    async fn create_session(
        &self,
        realm_id: Uuid,
        client_id: Uuid,
        redirect_uri: String,
        response_type: String,
        scope: String,
        state: Option<String>,
        nonce: Option<String>,
        user_id: Option<Uuid>,
    ) -> Result<AuthSession, AuthSessionError>;

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthSessionError>;
}

#[async_trait]
pub trait AuthSessionRepository: Send + Sync {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthSessionError>;
    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthSessionError>;
}
