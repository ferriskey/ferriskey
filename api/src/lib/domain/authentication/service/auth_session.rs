use async_trait::async_trait;

use uuid::Uuid;

use crate::domain::authentication::{
    entities::auth_session::{AuthSession, AuthSessionError},
    ports::auth_session::{AuthSessionRepository, AuthSessionService},
};

pub struct AuthSessionServiceImpl {
    pub repository: Box<dyn AuthSessionRepository>,
}

impl AuthSessionServiceImpl {
    pub fn new(repository: Box<dyn AuthSessionRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl AuthSessionService for AuthSessionServiceImpl {
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
    ) -> Result<AuthSession, AuthSessionError> {
        let session = AuthSession::new(
            realm_id,
            client_id,
            redirect_uri,
            response_type,
            scope,
            state,
            nonce,
            user_id,
        );
        self.repository.create(&session).await?;
        Ok(session)
    }
}
