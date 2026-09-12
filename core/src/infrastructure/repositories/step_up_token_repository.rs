use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        trident::ports::{StepUpTokenRecord, StepUpTokenRepository},
    },
    entity::step_up_tokens::{
        ActiveModel as SutActiveModel, Column as SutColumn, Entity as SutEntity,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresStepUpTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresStepUpTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl StepUpTokenRepository for PostgresStepUpTokenRepository {
    async fn save(&self, record: StepUpTokenRecord) -> Result<(), CoreError> {
        let active_model = SutActiveModel {
            id: Set(record.id),
            user_id: Set(record.user_id),
            token_hash: Set(record.token_hash),
            created_at: Set(Utc::now().fixed_offset()),
            expires_at: Set(record.expires_at.fixed_offset()),
        };

        SutEntity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to persist step-up token: {e}");
                CoreError::InternalServerError
            })?;
        Ok(())
    }

    async fn find_active(&self, user_id: Uuid) -> Result<Vec<StepUpTokenRecord>, CoreError> {
        let now = Utc::now().fixed_offset();

        let models = SutEntity::find()
            .filter(SutColumn::UserId.eq(user_id))
            .filter(SutColumn::ExpiresAt.gt(now))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to load active step-up tokens: {e}");
                CoreError::InternalServerError
            })?;

        Ok(models
            .into_iter()
            .map(|model| StepUpTokenRecord {
                id: model.id,
                user_id: model.user_id,
                token_hash: model.token_hash,
                expires_at: model.expires_at.into(),
            })
            .collect())
    }

    async fn delete_by_id(&self, token_id: Uuid) -> Result<bool, CoreError> {
        let now = Utc::now().fixed_offset();
        let result = SutEntity::delete_many()
            .filter(SutColumn::Id.eq(token_id))
            .filter(SutColumn::ExpiresAt.gt(now))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete step-up token: {e}");
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected > 0)
    }

    async fn cleanup_expired(&self) -> Result<u64, CoreError> {
        let now: DateTime<Utc> = Utc::now();
        let result = SutEntity::delete_many()
            .filter(SutColumn::ExpiresAt.lt(now.fixed_offset()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired step-up tokens: {e}");
                CoreError::InternalServerError
            })?;
        Ok(result.rows_affected)
    }
}
