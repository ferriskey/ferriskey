use crate::domain::authentication::entities::{AuthSession, AuthenticationError};
use crate::domain::authentication::ports::AuthSessionRepository;
use crate::infrastructure::repositories::auth_session_repository::PostgresAuthSessionRepository;
use uuid::Uuid;
use webauthn_rs::prelude::PasskeyRegistration;

#[derive(Clone)]
pub enum AuthSessionRepoAny {
    Postgres(PostgresAuthSessionRepository),
}

impl AuthSessionRepository for AuthSessionRepoAny {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.create(session).await,
        }
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.get_by_session_code(session_code).await,
        }
    }

    async fn get_by_code(&self, code: String) -> Result<Option<AuthSession>, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.get_by_code(code).await,
        }
    }

    async fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => {
                repo.update_code_and_user_id(session_code, code, user_id)
                    .await
            }
        }
    }

    async fn save_webauthn_challenge(
        &self,
        session_code: Uuid,
        challenge: &[u8],
    ) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => {
                repo.save_webauthn_challenge(session_code, challenge).await
            }
        }
    }

    async fn take_webauthn_challenge(
        &self,
        session_code: Uuid,
    ) -> Result<Option<PasskeyRegistration>, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.take_webauthn_challenge(session_code).await,
        }
    }
}
