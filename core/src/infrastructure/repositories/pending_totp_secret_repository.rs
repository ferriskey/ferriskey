use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        trident::ports::{PendingTotpSecretRecord, PendingTotpSecretRepository},
    },
    entity::pending_totp_secrets::{
        ActiveModel as PtsActiveModel, Column as PtsColumn, Entity as PtsEntity,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresPendingTotpSecretRepository {
    pub db: DatabaseConnection,
}

impl PostgresPendingTotpSecretRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PendingTotpSecretRepository for PostgresPendingTotpSecretRepository {
    async fn save(&self, record: PendingTotpSecretRecord) -> Result<(), CoreError> {
        let active_model = PtsActiveModel {
            user_id: Set(record.user_id),
            secret: Set(record.secret),
            label: Set(record.label),
            created_at: Set(Utc::now().fixed_offset()),
            expires_at: Set(record.expires_at.fixed_offset()),
        };

        // Upsert: a user can only have one pending TOTP secret.
        let on_conflict = OnConflict::columns([PtsColumn::UserId])
            .update_columns([
                PtsColumn::Secret,
                PtsColumn::Label,
                PtsColumn::CreatedAt,
                PtsColumn::ExpiresAt,
            ])
            .to_owned();

        PtsEntity::insert(active_model)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to persist pending TOTP secret: {e}");
                CoreError::InternalServerError
            })?;
        Ok(())
    }

    async fn take(&self, user_id: Uuid) -> Result<Option<PendingTotpSecretRecord>, CoreError> {
        let now = Utc::now().fixed_offset();

        // Atomically delete and return one unexpired secret. A single
        // DELETE ... RETURNING guarantees an expired secret is never consumed
        // and a concurrent request can never reuse the same row.
        let models = PtsEntity::delete_many()
            .filter(PtsColumn::UserId.eq(user_id))
            .filter(PtsColumn::ExpiresAt.gt(now))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to consume pending TOTP secret: {e}");
                CoreError::InternalServerError
            })?;

        let Some(model) = models.into_iter().next() else {
            return Ok(None);
        };

        Ok(Some(PendingTotpSecretRecord {
            user_id: model.user_id,
            secret: model.secret,
            label: model.label,
            expires_at: Utc::now(),
        }))
    }

    async fn cleanup_expired(&self) -> Result<u64, CoreError> {
        let now: DateTime<Utc> = Utc::now();
        let result = PtsEntity::delete_many()
            .filter(PtsColumn::ExpiresAt.lt(now.fixed_offset()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired pending TOTP secrets: {e}");
                CoreError::InternalServerError
            })?;
        Ok(result.rows_affected)
    }
}
