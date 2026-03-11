use std::sync::Arc;
use sqlx::{PgPool, query_as};
use uuid::Uuid;
use crate::domain::{
    common::entities::app_errors::CoreError,
    password_policy::{
        entities::{PasswordPolicy, UpdatePasswordPolicy},
        ports::PasswordPolicyRepository,
    },
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PostgresPasswordPolicyRepository {
    pool: PgPool,
}

impl PostgresPasswordPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Internal model for database mapping
#[derive(sqlx::FromRow)]
struct PasswordPolicyRow {
    id: Uuid,
    realm_id: Uuid,
    min_length: i32,
    require_uppercase: bool,
    require_lowercase: bool,
    require_number: bool,
    require_special: bool,
    max_age_days: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PasswordPolicyRow> for PasswordPolicy {
    fn from(row: PasswordPolicyRow) -> Self {
        Self {
            id: row.id,
            realm_id: row.realm_id.into(),
            min_length: row.min_length,
            require_uppercase: row.require_uppercase,
            require_lowercase: row.require_lowercase,
            require_number: row.require_number,
            require_special: row.require_special,
            max_age_days: row.max_age_days,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl PasswordPolicyRepository for PostgresPasswordPolicyRepository {
    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Option<PasswordPolicy>, CoreError> {
        let row: Option<PasswordPolicyRow> = query_as!(
            PasswordPolicyRow,
            r#"
            SELECT id, realm_id, min_length, require_uppercase, require_lowercase, require_number, require_special, max_age_days, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            FROM password_policy
            WHERE realm_id = $1
            "#,
            realm_id as Uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch password policy: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(row.map(Into::into))
    }

    async fn upsert(&self, realm_id: Uuid, update: UpdatePasswordPolicy) -> Result<PasswordPolicy, CoreError> {
        let now = Utc::now();
        
        let existing = self.find_by_realm_id(realm_id).await?;
        
        let row: PasswordPolicyRow = if let Some(policy) = existing {
            // Update
            query_as!(
                PasswordPolicyRow,
                r#"
                UPDATE password_policy
                SET min_length = $1, require_uppercase = $2, require_lowercase = $3, require_number = $4, require_special = $5, max_age_days = $6, updated_at = $7
                WHERE realm_id = $8
                RETURNING id, realm_id, min_length, require_uppercase, require_lowercase, require_number, require_special, max_age_days, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
                "#,
                update.min_length.unwrap_or(policy.min_length) as i32,
                update.require_uppercase.unwrap_or(policy.require_uppercase) as bool,
                update.require_lowercase.unwrap_or(policy.require_lowercase) as bool,
                update.require_number.unwrap_or(policy.require_number) as bool,
                update.require_special.unwrap_or(policy.require_special) as bool,
                update.max_age_days.or(policy.max_age_days) as Option<i32>,
                now as DateTime<Utc>,
                realm_id as Uuid
            )
            .fetch_one(&self.pool)
            .await
        } else {
            // Insert (should not happen if migration added default)
            query_as!(
                PasswordPolicyRow,
                r#"
                INSERT INTO password_policy (id, realm_id, min_length, require_uppercase, require_lowercase, require_number, require_special, max_age_days, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING id, realm_id, min_length, require_uppercase, require_lowercase, require_number, require_special, max_age_days, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
                "#,
                Uuid::now_v7() as Uuid,
                realm_id as Uuid,
                update.min_length.unwrap_or(8) as i32,
                update.require_uppercase.unwrap_or(false) as bool,
                update.require_lowercase.unwrap_or(false) as bool,
                update.require_number.unwrap_or(false) as bool,
                update.require_special.unwrap_or(false) as bool,
                update.max_age_days as Option<i32>,
                now as DateTime<Utc>,
                now as DateTime<Utc>
            )
            .fetch_one(&self.pool)
            .await
        }.map_err(|e| {
            tracing::error!("Failed to upsert password policy: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(row.into())
    }
}
