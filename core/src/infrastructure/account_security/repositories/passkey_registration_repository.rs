use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use uuid::Uuid;
use webauthn_rs::prelude::PasskeyRegistration;

use crate::domain::account_security::ports::PasskeyRegistrationRepository;
use crate::domain::common::{entities::app_errors::CoreError, generate_uuid_v7};
use crate::entity::passkey_registrations::{
    ActiveModel, Column, Entity as PasskeyRegistrationEntity,
};

#[derive(Debug, Clone)]
pub struct PostgresPasskeyRegistrationRepository {
    pub db: DatabaseConnection,
}

impl PostgresPasskeyRegistrationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PasskeyRegistrationRepository for PostgresPasskeyRegistrationRepository {
    async fn start(
        &self,
        user_id: Uuid,
        registration: PasskeyRegistration,
        expires_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let state = serde_json::to_value(&registration).map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to serialise a passkey challenge: {e:?}");
            CoreError::InternalServerError
        })?;

        self.clear_for_user(user_id).await?;

        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            user_id: Set(user_id),
            state: Set(state),
            expires_at: Set(expires_at.naive_utc()),
            consumed_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
        };

        model.insert(&self.db).await.map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to record a passkey challenge: {e:?}");
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn consume(
        &self,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<PasskeyRegistration>, CoreError> {
        let naive_now = now.naive_utc();

        let candidate = PasskeyRegistrationEntity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ConsumedAt.is_null())
            .filter(Column::ExpiresAt.gt(naive_now))
            .order_by_desc(Column::CreatedAt)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to load a passkey challenge: {e:?}");
                CoreError::InternalServerError
            })?;

        let Some(candidate) = candidate else {
            return Ok(None);
        };

        let claimed = PasskeyRegistrationEntity::update_many()
            .col_expr(Column::ConsumedAt, Expr::value(naive_now))
            .filter(Column::Id.eq(candidate.id))
            .filter(Column::ConsumedAt.is_null())
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to consume a passkey challenge: {e:?}");
                CoreError::InternalServerError
            })?;

        if claimed.rows_affected == 0 {
            return Ok(None);
        }

        let registration = serde_json::from_value(candidate.state).map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to read back a passkey challenge: {e:?}");
            CoreError::InternalServerError
        })?;

        Ok(Some(registration))
    }
}

impl PostgresPasskeyRegistrationRepository {
    async fn clear_for_user(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let deleted = PasskeyRegistrationEntity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to clear passkey challenges: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(deleted.rows_affected)
    }
}
