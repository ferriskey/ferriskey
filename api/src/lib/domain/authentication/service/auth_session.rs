use uuid::Uuid;

use crate::{
    domain::authentication::{
        entities::{
            auth_session::{AuthSession, AuthSessionError, AuthSessionParams},
            dto::CreateAuthSessionDto,
        },
        ports::auth_session::{AuthSessionRepository, AuthSessionService},
    },
    infrastructure::repositories::auth_session_repository::PostgresAuthSessionRepository,
};

pub type DefaultAuthSessionService = AuthSessionServiceImpl<PostgresAuthSessionRepository>;

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
        dto: CreateAuthSessionDto,
        // realm_id: Uuid,
        // client_id: Uuid,
        // redirect_uri: String,
        // response_type: String,
        // scope: String,
        // state: Option<String>,
        // nonce: Option<String>,
        // user_id: Option<Uuid>,
    ) -> Result<AuthSession, AuthSessionError> {
        let params = AuthSessionParams {
            realm_id: dto.realm_id,
            client_id: dto.client_id,
            redirect_uri: dto.redirect_uri,
            response_type: dto.response_type,
            scope: dto.scope,
            state: dto.state,
            nonce: dto.nonce,
            user_id: dto.user_id,
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
    ) -> Result<AuthSession, AuthSessionError> {
        self.repository.get_by_session_code(session_code).await
    }

    async fn get_by_code(&self, code: String) -> Result<AuthSession, AuthSessionError> {
        self.repository
            .get_by_code(code)
            .await?
            .ok_or(AuthSessionError::NotFound)
    }

    async fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthSessionError> {
        self.repository
            .update_code_and_user_id(session_code, code, user_id)
            .await
    }
}
