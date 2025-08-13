use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{
        entities::{JwtError, RefreshToken},
        ports::RefreshTokenRepository,
    },
};

impl From<crate::entity::refresh_tokens::Model> for RefreshToken {
    fn from(model: crate::entity::refresh_tokens::Model) -> Self {
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
    pub db: DatabaseConnection,
}

impl PostgresRefreshTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<RefreshToken, JwtError> {
        let model = crate::entity::refresh_tokens::ActiveModel {
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

    async fn get_by_jti(&self, jti: Uuid) -> Result<RefreshToken, JwtError> {
        let refresh_token = crate::entity::refresh_tokens::Entity::find()
            .filter(crate::entity::refresh_tokens::Column::Jti.eq(jti))
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?
            .ok_or_else(|| JwtError::GenerationError("Refresh token not found".to_string()))?;

        Ok(refresh_token.into())
    }

    async fn delete(&self, jti: Uuid) -> Result<(), JwtError> {
        crate::entity::refresh_tokens::Entity::delete_many()
            .filter(crate::entity::refresh_tokens::Column::Jti.eq(jti))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}
