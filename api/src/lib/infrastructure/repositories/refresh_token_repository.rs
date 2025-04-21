use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    jwt::{
        entities::{jwt_error::JwtError, refresh_token::RefreshToken},
        ports::jwt_repository::RefreshTokenRepository,
    },
    utils::generate_uuid_v7,
};

#[derive(Debug, Clone)]
pub struct PostgresRefreshTokenRepository {
    pub pool: PgPool,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<RefreshToken, JwtError> {
        sqlx::query_as!(
            RefreshToken,
            "INSERT INTO refresh_tokens (id, jti, user_id, revoked, expires_at) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            generate_uuid_v7(),
            jti,
            user_id,
            false,
            expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| JwtError::GenerationError(e.to_string()))
    }

    async fn delete(&self, jti: Uuid) -> Result<(), JwtError> {
        sqlx::query!("DELETE FROM refresh_tokens WHERE jti = $1", jti)
            .execute(&self.pool)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}
