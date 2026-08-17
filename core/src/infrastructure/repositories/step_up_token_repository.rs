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

    async fn take(&self, user_id: Uuid) -> Result<Option<String>, CoreError> {
        let now = Utc::now().fixed_offset();

        // Atomically delete and return one unexpired token. A single
        // DELETE ... RETURNING guarantees an expired token is never consumed
        // and a concurrent request can never reuse the same row.
        let models = SutEntity::delete_many()
            .filter(SutColumn::UserId.eq(user_id))
            .filter(SutColumn::ExpiresAt.gt(now))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to consume step-up token: {e}");
                CoreError::InternalServerError
            })?;

        let Some(model) = models.into_iter().next() else {
            return Ok(None);
        };

        Ok(Some(model.token_hash))
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
