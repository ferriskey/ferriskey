use chrono::{DateTime, Utc};
use sea_orm::{
    sea_query::OnConflict, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        authentication::entities::WebAuthnChallenge,
        common::entities::app_errors::CoreError,
        trident::ports::WebAuthnChallengeRepository,
    },
    entity::webauthn_challenges::{
        ActiveModel as WcActiveModel, Column as WcColumn, Entity as WcEntity,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresWebAuthnChallengeRepository {
    pub db: DatabaseConnection,
}

impl PostgresWebAuthnChallengeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl WebAuthnChallengeRepository for PostgresWebAuthnChallengeRepository {
    async fn save(
        &self,
        record: crate::domain::trident::ports::WebAuthnChallengeRecord,
    ) -> Result<(), CoreError> {
        let challenge = serde_json::to_value(&record.challenge).map_err(|e| {
            error!("Failed to serialize WebAuthn challenge: {e}");
            CoreError::InternalServerError
        })?;

        let active_model = WcActiveModel {
            user_id: Set(record.user_id),
            challenge: Set(challenge),
            created_at: Set(Utc::now().fixed_offset()),
            expires_at: Set(record.expires_at.fixed_offset()),
        };

        // Upsert: a user can only have one pending registration challenge.
        let on_conflict = OnConflict::columns([WcColumn::UserId])
            .update_columns([
                WcColumn::Challenge,
                WcColumn::CreatedAt,
                WcColumn::ExpiresAt,
            ])
            .to_owned();

        WcEntity::insert(active_model)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to persist WebAuthn challenge: {e}");
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn take(&self, user_id: Uuid) -> Result<Option<WebAuthnChallenge>, CoreError> {
        let model = WcEntity::find()
            .filter(WcColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to load WebAuthn challenge: {e}");
                CoreError::InternalServerError
            })?;

        let Some(model) = model else {
            return Ok(None);
        };

        let challenge: WebAuthnChallenge = serde_json::from_value(model.challenge).map_err(|e| {
            error!("Failed to deserialize WebAuthn challenge: {e}");
            CoreError::InternalServerError
        })?;

        // Consume the challenge so it cannot be reused.
        WcEntity::delete_many()
            .filter(WcColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete WebAuthn challenge: {e}");
                CoreError::InternalServerError
            })?;

        Ok(Some(challenge))
    }

    async fn cleanup_expired(&self) -> Result<u64, CoreError> {
        let now: DateTime<Utc> = Utc::now();
        let result = WcEntity::delete_many()
            .filter(WcColumn::ExpiresAt.lt(now.fixed_offset()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired WebAuthn challenges: {e}");
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}
