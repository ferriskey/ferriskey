use crate::domain::authentication::entities::{AuthSession, AuthenticationError};
use crate::domain::authentication::ports::AuthSessionRepository;
use crate::infrastructure::repositories::auth_session_repository::PostgresAuthSessionRepository;
use uuid::Uuid;

#[cfg(test)]
use crate::domain::authentication::ports::test::MockAuthSessionRepository;

#[derive(Clone)]
pub enum AuthSessionRepoAny {
    Postgres(PostgresAuthSessionRepository),
    #[cfg(test)]
    Mock(MockAuthSessionRepository),
}

impl AuthSessionRepository for AuthSessionRepoAny {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.create(session).await,
            #[cfg(test)]
            AuthSessionRepoAny::Mock(m) => m.create(session).await,
        }
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.get_by_session_code(session_code).await,
            #[cfg(test)]
            AuthSessionRepoAny::Mock(m) => m.get_by_session_code(session_code).await,
        }
    }

    async fn get_by_code(&self, code: String) -> Result<Option<AuthSession>, AuthenticationError> {
        match self {
            AuthSessionRepoAny::Postgres(repo) => repo.get_by_code(code).await,
            #[cfg(test)]
            AuthSessionRepoAny::Mock(m) => m.get_by_code(code).await,
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
            #[cfg(test)]
            AuthSessionRepoAny::Mock(m) => m.update_code_and_user_id(session_code, code, user_id).await,
        }
    }
}
