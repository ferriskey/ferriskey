use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use uuid::Uuid;

use crate::domain::{
    common::{entities::app_errors::CoreError, generate_uuid_v7},
    trident::ports::{OtpEnrollment, OtpEnrollmentRepository},
};
use crate::entity::otp_enrollments::{ActiveModel, Column, Entity as OtpEnrollmentEntity, Model};

#[derive(Debug, Clone)]
pub struct PostgresOtpEnrollmentRepository {
    pub db: DatabaseConnection,
}

impl PostgresOtpEnrollmentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: Model) -> OtpEnrollment {
    OtpEnrollment {
        id: model.id,
        user_id: model.user_id,
        secret: model.secret,
        expires_at: model.expires_at.and_utc(),
        created_at: model.created_at.and_utc(),
    }
}

impl OtpEnrollmentRepository for PostgresOtpEnrollmentRepository {
    async fn start_enrollment(
        &self,
        user_id: Uuid,
        secret: String,
        expires_at: DateTime<Utc>,
    ) -> Result<OtpEnrollment, CoreError> {
        // Restarting setup invalidates whatever was pending: leaving the previous
        // candidate live would let an abandoned secret stay enrollable until its TTL.
        self.clear_enrollments(user_id).await?;

        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            user_id: Set(user_id),
            secret: Set(secret),
            expires_at: Set(expires_at.naive_utc()),
            consumed_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
        };

        let model = model.insert(&self.db).await.map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to start OTP enrollment: {e:?}");
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn consume_enrollment(
        &self,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<OtpEnrollment>, CoreError> {
        let naive_now = now.naive_utc();

        let candidate = OtpEnrollmentEntity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ConsumedAt.is_null())
            .filter(Column::ExpiresAt.gt(naive_now))
            .order_by_desc(Column::CreatedAt)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to load OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?;

        let Some(candidate) = candidate else {
            return Ok(None);
        };

        // Compare-and-swap: the `ConsumedAt.is_null()` predicate is what makes this
        // single-use. A concurrent call that lost the race updates 0 rows and gets
        // `None`, rather than both callers enrolling the same secret.
        let claimed = OtpEnrollmentEntity::update_many()
            .col_expr(Column::ConsumedAt, Expr::value(naive_now))
            .filter(Column::Id.eq(candidate.id))
            .filter(Column::ConsumedAt.is_null())
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to consume OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?;

        if claimed.rows_affected == 0 {
            return Ok(None);
        }

        Ok(Some(model_to_domain(candidate)))
    }

    async fn clear_enrollments(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let deleted = OtpEnrollmentEntity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to clear OTP enrollments: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(deleted.rows_affected)
    }
}
