use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    jwt::{
        entities::{jwt_error::JwtError, refresh_token::RefreshToken},
        ports::jwt_repository::RefreshTokenRepository,
    },
    utils::generate_uuid_v7,
};

impl From<entity::refresh_tokens::Model> for RefreshToken {
    fn from(model: entity::refresh_tokens::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = model.expires_at.map(|dt| Utc.from_utc_datetime(&dt));

        RefreshToken {
            id: model.id,
            jti: model.jti,
            user_id: model.user_id,
            revoked: model.revoked,
            created_at,
            expires_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresRefreshTokenRepository {
    pub pool: PgPool,
    pub db: DatabaseConnection,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: PgPool, db: DatabaseConnection) -> Self {
        Self { pool, db }
    }
}

impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<RefreshToken, JwtError> {
        let model = entity::refresh_tokens::ActiveModel {
            id: Set(generate_uuid_v7()),
            jti: Set(jti),
            user_id: Set(user_id),
            revoked: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
        };

        let refresh_token = model
            .insert(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(refresh_token.into())
    }

    async fn delete(&self, jti: Uuid) -> Result<(), JwtError> {
        entity::refresh_tokens::Entity::delete_many()
            .filter(entity::refresh_tokens::Column::Jti.eq(jti))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}
