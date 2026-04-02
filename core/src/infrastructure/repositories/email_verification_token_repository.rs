use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        email_verification::{
            entities::EmailVerificationToken,
            ports::{CreateEmailVerificationTokenInput, EmailVerificationTokenRepository},
        },
    },
    entity::email_verification_tokens::{
        ActiveModel as EvtActiveModel, Column as EvtColumn, Entity as EvtEntity, Model as EvtModel,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresEmailVerificationTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresEmailVerificationTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<EvtModel> for EmailVerificationToken {
    fn from(model: EvtModel) -> Self {
        let created_at: DateTime<Utc> = model.created_at.into();
        let expires_at: DateTime<Utc> = model.expires_at.into();
        let used_at: Option<DateTime<Utc>> = model.used_at.map(|t| t.into());

        EmailVerificationToken {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            token_hash: model.token_hash,
            created_at,
            expires_at,
            used_at,
        }
    }
}

impl EmailVerificationTokenRepository for PostgresEmailVerificationTokenRepository {
    async fn create(
        &self,
        input: CreateEmailVerificationTokenInput,
    ) -> Result<EmailVerificationToken, CoreError> {
        let id = Uuid::new_v4();
        let active_model = EvtActiveModel {
            id: Set(id),
            user_id: Set(input.user_id),
            realm_id: Set(input.realm_id),
            token_hash: Set(input.token_hash),
            expires_at: Set(input.expires_at.fixed_offset()),
            created_at: Set(Utc::now().fixed_offset()),
            used_at: Set(None),
        };

        let model = active_model.insert(&self.db).await.map_err(|e| {
            error!("Failed to create email verification token: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn find_valid_by_hash(
        &self,
        token_hash: &str,
        realm_id: Uuid,
    ) -> Result<Option<EmailVerificationToken>, CoreError> {
        let now = Utc::now().fixed_offset();
        let model = EvtEntity::find()
            .filter(EvtColumn::TokenHash.eq(token_hash))
            .filter(EvtColumn::RealmId.eq(realm_id))
            .filter(EvtColumn::UsedAt.is_null())
            .filter(EvtColumn::ExpiresAt.gt(now))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find email verification token: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(EmailVerificationToken::from))
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), CoreError> {
        let active_model = EvtActiveModel {
            id: Set(id),
            used_at: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        };

        EvtEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to mark email verification token as used: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let result = EvtEntity::delete_many()
            .filter(EvtColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete email verification tokens: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}
