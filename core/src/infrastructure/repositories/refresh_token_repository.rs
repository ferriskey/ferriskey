use chrono::{DateTime, TimeZone, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{
        JwtError,
        entities::{RefreshToken, RefreshTokenStatus},
        ports::{RefreshTokenRepository, RotateOutcome},
    },
};

impl From<crate::entity::refresh_tokens::Model> for RefreshToken {
    fn from(model: crate::entity::refresh_tokens::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = model.expires_at.map(|dt| Utc.from_utc_datetime(&dt));
        let rotated_at = model.rotated_at.map(|dt| dt.with_timezone(&Utc));
        let status = RefreshTokenStatus::parse(&model.status);

        RefreshToken {
            id: model.id,
            jti: model.jti,
            user_id: model.user_id,
            revoked: model.revoked,
            created_at,
            expires_at,
            family_id: model.family_id,
            status,
            replaced_by: model.replaced_by,
            rotated_at,
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
        let family_id = Uuid::new_v4();
        let model = crate::entity::refresh_tokens::ActiveModel {
            id: Set(generate_uuid_v7()),
            jti: Set(jti),
            user_id: Set(user_id),
            revoked: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
            family_id: Set(family_id),
            status: Set("active".to_string()),
            replaced_by: Set(None),
            rotated_at: Set(None),
        };

        let refresh_token = model
            .insert(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(refresh_token.into())
    }

    async fn create_in_family(
        &self,
        jti: Uuid,
        user_id: Uuid,
        family_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<RefreshToken, JwtError> {
        let model = crate::entity::refresh_tokens::ActiveModel {
            id: Set(generate_uuid_v7()),
            jti: Set(jti),
            user_id: Set(user_id),
            revoked: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
            family_id: Set(family_id),
            status: Set("active".to_string()),
            replaced_by: Set(None),
            rotated_at: Set(None),
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
            .ok_or(JwtError::InvalidToken)?;

        Ok(refresh_token.into())
    }

    async fn revoke_by_jti(&self, jti: Uuid) -> Result<(), JwtError> {
        crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Revoked,
                Expr::value(true),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("revoked"),
            )
            .filter(crate::entity::refresh_tokens::Column::Jti.eq(jti))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, jti: Uuid) -> Result<(), JwtError> {
        crate::entity::refresh_tokens::Entity::delete_many()
            .filter(crate::entity::refresh_tokens::Column::Jti.eq(jti))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }

    async fn rotate(
        &self,
        old_id: Uuid,
        new_jti: Uuid,
        user_id: Uuid,
        family_id: Uuid,
        new_expires_at: Option<DateTime<Utc>>,
    ) -> Result<RotateOutcome, JwtError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        let new_id = generate_uuid_v7();
        let now = Utc::now();

        // Conditional UPDATE: only succeeds when this token is still active.
        let result = crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("rotated"),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::ReplacedBy,
                Expr::value(new_id),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::RotatedAt,
                Expr::value(now.fixed_offset()),
            )
            .filter(crate::entity::refresh_tokens::Column::Id.eq(old_id))
            .filter(crate::entity::refresh_tokens::Column::Status.eq("active"))
            .exec(&txn)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        if result.rows_affected == 0 {
            txn.rollback()
                .await
                .map_err(|e| JwtError::GenerationError(e.to_string()))?;
            return Ok(RotateOutcome::Conflict);
        }

        // Mint the successor inside the same transaction.
        let new_model = crate::entity::refresh_tokens::ActiveModel {
            id: Set(new_id),
            jti: Set(new_jti),
            user_id: Set(user_id),
            revoked: Set(false),
            created_at: Set(now.naive_utc()),
            expires_at: Set(new_expires_at.map(|dt| dt.naive_utc())),
            family_id: Set(family_id),
            status: Set("active".to_string()),
            replaced_by: Set(None),
            rotated_at: Set(None),
        };

        let new_token = new_model
            .insert(&txn)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(RotateOutcome::Rotated(new_token.into()))
    }

    async fn revoke_family(&self, family_id: Uuid) -> Result<(), JwtError> {
        crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("revoked"),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(crate::entity::refresh_tokens::Column::FamilyId.eq(family_id))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}
