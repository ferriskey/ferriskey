use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::authentication::device_flow::entities::{
    DeviceAuthSession, DeviceAuthStatus, UserCode,
};
use crate::domain::authentication::device_flow::ports::DeviceAuthRepository;
use crate::domain::authentication::entities::AuthenticationError;
use crate::entity::device_auth_sessions::{
    ActiveModel as DasActiveModel, Column as DasColumn, Entity as DasEntity, Model as DasModel,
};

impl From<DasModel> for DeviceAuthSession {
    fn from(model: DasModel) -> Self {
        let created_at: DateTime<Utc> = model.created_at.into();
        let expires_at: DateTime<Utc> = model.expires_at.into();
        let last_polled_at: Option<DateTime<Utc>> = model.last_polled_at.map(Into::into);

        DeviceAuthSession {
            // The schema uses `device_code` as the primary key; there is no
            // separate `id` column, so the two coincide once persisted.
            id: model.device_code,
            realm_id: model.realm_id.into(),
            client_id: model.client_id,
            device_code: model.device_code,
            user_code: UserCode::new(model.user_code),
            scope: model.scope,
            status: DeviceAuthStatus::from_db_value(&model.status)
                .unwrap_or(DeviceAuthStatus::Pending),
            user_id: model.user_id,
            interval: i64::from(model.interval_seconds),
            created_at,
            expires_at,
            last_polled_at,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresDeviceAuthRepository {
    pub db: DatabaseConnection,
}

impl PostgresDeviceAuthRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Delete all sessions whose lifetime has elapsed. Returns the number of
    /// rows removed. Intended to be run periodically by a background job.
    #[allow(dead_code)]
    pub async fn purge_expired(&self) -> Result<u64, AuthenticationError> {
        let now = Utc::now().fixed_offset();
        let result = DasEntity::delete_many()
            .filter(DasColumn::ExpiresAt.lt(now))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error purging expired device auth sessions: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}

impl DeviceAuthRepository for PostgresDeviceAuthRepository {
    async fn create(
        &self,
        session: &DeviceAuthSession,
    ) -> Result<DeviceAuthSession, AuthenticationError> {
        let model = DasActiveModel {
            device_code: Set(session.device_code),
            user_code: Set(session.user_code.as_str().to_string()),
            client_id: Set(session.client_id),
            realm_id: Set(session.realm_id.into()),
            user_id: Set(session.user_id),
            scope: Set(session.scope.clone()),
            status: Set(session.status.as_str().to_string()),
            interval_seconds: Set(session.interval as i32),
            expires_at: Set(session.expires_at.fixed_offset()),
            last_polled_at: Set(session.last_polled_at.map(|d| d.fixed_offset())),
            created_at: Set(session.created_at.fixed_offset()),
        };

        let model = model.insert(&self.db).await.map_err(|e| {
            error!("Error creating device auth session: {e:?}");
            AuthenticationError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn find_by_device_code(
        &self,
        device_code: Uuid,
    ) -> Result<Option<DeviceAuthSession>, AuthenticationError> {
        let model = DasEntity::find_by_id(device_code)
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding device auth session by device_code: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(model.map(Into::into))
    }

    async fn find_by_user_code(
        &self,
        user_code: String,
    ) -> Result<Option<DeviceAuthSession>, AuthenticationError> {
        let model = DasEntity::find()
            .filter(DasColumn::UserCode.eq(user_code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding device auth session by user_code: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(model.map(Into::into))
    }

    async fn update_status(
        &self,
        device_code: Uuid,
        status: DeviceAuthStatus,
        user_id: Option<Uuid>,
    ) -> Result<DeviceAuthSession, AuthenticationError> {
        let mut update = DasEntity::update_many()
            .col_expr(DasColumn::Status, Expr::value(status.as_str().to_string()));

        if let Some(user_id) = user_id {
            update = update.col_expr(DasColumn::UserId, Expr::value(user_id));
        }

        let model = update
            .filter(DasColumn::DeviceCode.eq(device_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating device auth session status: {e:?}");
                AuthenticationError::InternalServerError
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?;

        Ok(model.into())
    }

    async fn mark_polled(&self, device_code: Uuid) -> Result<(), AuthenticationError> {
        DasEntity::update_many()
            .col_expr(
                DasColumn::LastPolledAt,
                Expr::value(Utc::now().fixed_offset()),
            )
            .filter(DasColumn::DeviceCode.eq(device_code))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error marking device auth session as polled: {e:?}");
                AuthenticationError::InternalServerError
            })?;

        Ok(())
    }
}
