use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        trident::{entities::PasswordResetToken, ports::PasswordResetTokenRepository},
    },
    entity::password_reset_tokens::{
        ActiveModel as PrtActiveModel, Column as PrtColumn, Entity as PrtEntity, Model as PrtModel,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresPasswordResetTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresPasswordResetTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<PrtModel> for PasswordResetToken {
    fn from(model: PrtModel) -> Self {
        let created_at: DateTime<Utc> = model.created_at.into();
        let expires_at: DateTime<Utc> = model.expires_at.into();

        PasswordResetToken {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            token_id: model.token_id,
            token_hash: model.token_hash,
            created_at,
            expires_at,
        }
    }
}

impl PasswordResetTokenRepository for PostgresPasswordResetTokenRepository {
    async fn create(&self, token: &PasswordResetToken) -> Result<(), CoreError> {
        let active_model = PrtActiveModel {
            id: Set(token.id),
            user_id: Set(token.user_id),
            realm_id: Set(token.realm_id),
            token_id: Set(token.token_id),
            token_hash: Set(token.token_hash.clone()),
            created_at: Set(token.created_at.fixed_offset()),
            expires_at: Set(token.expires_at.fixed_offset()),
        };

        active_model.insert(&self.db).await.map_err(|e| {
            error!("Failed to create password reset token: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn get_by_token_id(
        &self,
        token_id: Uuid,
    ) -> Result<Option<PasswordResetToken>, CoreError> {
        let result = PrtEntity::find()
            .filter(PrtColumn::TokenId.eq(token_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get password reset token by token_id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.map(PasswordResetToken::from))
    }

    async fn delete_by_token_id(&self, token_id: Uuid) -> Result<(), CoreError> {
        PrtEntity::delete_many()
            .filter(PrtColumn::TokenId.eq(token_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete password reset token by token_id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn delete_all_by_user_id(&self, user_id: Uuid) -> Result<(), CoreError> {
        PrtEntity::delete_many()
            .filter(PrtColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete password reset tokens by user_id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn count_active_by_user_id(&self, user_id: Uuid) -> Result<i64, CoreError> {
        use sea_orm::PaginatorTrait;

        let now = Utc::now().fixed_offset();
        let count = PrtEntity::find()
            .filter(PrtColumn::UserId.eq(user_id))
            .filter(PrtColumn::ExpiresAt.gt(now))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to count active password reset tokens: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(count as i64)
    }

    async fn cleanup_expired(&self) -> Result<u64, CoreError> {
        let now = Utc::now().fixed_offset();
        let result = PrtEntity::delete_many()
            .filter(PrtColumn::ExpiresAt.lt(now))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired password reset tokens: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}
