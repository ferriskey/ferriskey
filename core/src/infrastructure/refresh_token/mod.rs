use crate::domain::jwt::entities::{JwtError, RefreshToken};
use crate::domain::jwt::ports::RefreshTokenRepository;
use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub enum RefreshTokenRepoAny {
    Postgres(PostgresRefreshTokenRepository),
}

impl RefreshTokenRepository for RefreshTokenRepoAny {
    async fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<RefreshToken, JwtError> {
        match self {
            RefreshTokenRepoAny::Postgres(repo) => repo.create(jti, user_id, expires_at).await,
        }
    }

    async fn get_by_jti(&self, jti: Uuid) -> Result<RefreshToken, JwtError> {
        match self {
            RefreshTokenRepoAny::Postgres(repo) => repo.get_by_jti(jti).await,
        }
    }

    async fn delete(&self, jti: Uuid) -> Result<(), JwtError> {
        match self {
            RefreshTokenRepoAny::Postgres(repo) => repo.delete(jti).await,
        }
    }
}
