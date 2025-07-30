use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthSessionParams, AuthenticationError},
    ports::{AuthSessionRepository, AuthSessionService},
    value_objects::CreateAuthSessionRequest,
};

#[derive(Clone)]
pub struct AuthSessionServiceImpl<R: AuthSessionRepository> {
    pub repository: R,
}

impl<R: AuthSessionRepository> AuthSessionServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: AuthSessionRepository> AuthSessionService for AuthSessionServiceImpl<R> {
    async fn create_session(
        &self,
        request: CreateAuthSessionRequest,
    ) -> Result<AuthSession, AuthenticationError> {
        let params = AuthSessionParams {
            realm_id: request.realm_id,
            client_id: request.client_id,
            redirect_uri: request.redirect_uri,
            response_type: request.response_type,
            scope: request.scope,
            state: request.state,
            nonce: request.nonce,
            user_id: request.user_id,
            code: None,
            authenticated: false,
        };
        let session = AuthSession::new(params);
        self.repository.create(&session).await?;
        Ok(session)
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        self.repository.get_by_session_code(session_code).await
    }

    async fn get_by_code(&self, code: String) -> Result<AuthSession, AuthenticationError> {
        self.repository
            .get_by_code(code)
            .await?
            .ok_or(AuthenticationError::NotFound)
    }

    async fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        self.repository
            .update_code_and_user_id(session_code, code, user_id)
            .await
    }
}
