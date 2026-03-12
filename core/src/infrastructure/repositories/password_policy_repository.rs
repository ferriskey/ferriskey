use chrono::Utc;
use sqlx::PgPool;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::password_policy::entity::{PasswordPolicy, UpdatePasswordPolicy};
use crate::domain::password_policy::repository::PasswordPolicyRepository;

#[derive(Debug, Clone)]
pub struct PostgresPasswordPolicyRepository {
    pub db: PgPool,
}

impl PostgresPasswordPolicyRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

impl PasswordPolicyRepository for PostgresPasswordPolicyRepository {
    #[instrument(skip(self), err)]
    async fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> Result<Option<PasswordPolicy>, CoreError> {
        let result = sqlx::query_as!(
            PasswordPolicy,
            r#"
            SELECT id, realm_id, min_length, require_uppercase, require_lowercase,
                   require_number, require_special, max_age_days, created_at, updated_at
            FROM password_policy
            WHERE realm_id = $1
            "#,
            realm_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to find password policy by realm_id: {}", e);
            CoreError::Database(e.to_string())
        })?;

        Ok(result)
    }

    #[instrument(skip(self, update), err)]
    async fn upsert(
        &self,
        realm_id: Uuid,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        let now = Utc::now();

        let result = sqlx::query_as!(
            PasswordPolicy,
            r#"
            INSERT INTO password_policy (
                id, realm_id, min_length, require_uppercase, require_lowercase,
                require_number, require_special, max_age_days, created_at, updated_at
            )
            VALUES (
                gen_random_uuid(), $1, COALESCE($2, 8), COALESCE($3, false),
                COALESCE($4, false), COALESCE($5, false), COALESCE($6, false), $7, $8, $8
            )
            ON CONFLICT (realm_id)
            DO UPDATE SET
                min_length = COALESCE($2, password_policy.min_length),
                require_uppercase = COALESCE($3, password_policy.require_uppercase),
                require_lowercase = COALESCE($4, password_policy.require_lowercase),
                require_number = COALESCE($5, password_policy.require_number),
                require_special = COALESCE($6, password_policy.require_special),
                max_age_days = COALESCE($7, password_policy.max_age_days),
                updated_at = $8
            RETURNING id, realm_id, min_length, require_uppercase, require_lowercase,
                      require_number, require_special, max_age_days, created_at, updated_at
            "#,
            realm_id,
            update.min_length,
            update.require_uppercase,
            update.require_lowercase,
            update.require_number,
            update.require_special,
            update.max_age_days,
            now
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to upsert password policy: {}", e);
            CoreError::Database(e.to_string())
        })?;

        Ok(result)
    }
}
