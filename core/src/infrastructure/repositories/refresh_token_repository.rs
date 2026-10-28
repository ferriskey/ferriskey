use chrono::{DateTime, TimeZone, Utc};
use sea_orm::sea_query::{Expr, Query};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DbBackend, EntityTrait, FromQueryResult, QueryFilter, Statement,
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
            resource: model.resource,
        }
    }
}

const ROTATE_SQL: &str = "\
WITH superseded AS (
    UPDATE refresh_tokens
    SET status = 'rotated', replaced_by = $2, rotated_at = $3
    WHERE id = $1 AND status = 'active'
    RETURNING session_id, resource
)
INSERT INTO refresh_tokens
    (id, jti, user_id, revoked, created_at, expires_at, family_id, status, session_id, resource)
SELECT $2, $4, $5, false, $6, $7, $8, 'active', session_id, resource
FROM superseded
RETURNING id, jti, user_id, revoked, expires_at, created_at, family_id, status, replaced_by, rotated_at, session_id, resource";

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
        session_id: Option<Uuid>,
        resource: Option<String>,
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
            session_id: Set(session_id),
            resource: Set(resource),
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
        session_id: Option<Uuid>,
        resource: Option<String>,
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
            session_id: Set(session_id),
            resource: Set(resource),
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
        let new_id = generate_uuid_v7();
        let now = Utc::now();

        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                ROTATE_SQL,
                [
                    old_id.into(),
                    new_id.into(),
                    now.fixed_offset().into(),
                    new_jti.into(),
                    user_id.into(),
                    now.naive_utc().into(),
                    new_expires_at.map(|dt| dt.naive_utc()).into(),
                    family_id.into(),
                ],
            ))
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(RotateOutcome::Conflict);
        };

        let successor = crate::entity::refresh_tokens::Model::from_query_result(&row, "")
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(RotateOutcome::Rotated(successor.into()))
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

    async fn revoke_by_session_id(&self, session_id: Uuid) -> Result<u64, JwtError> {
        let result = crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("revoked"),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(crate::entity::refresh_tokens::Column::SessionId.eq(session_id))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn delete_expired(&self, before: DateTime<Utc>, limit: u64) -> Result<u64, JwtError> {
        let oldest = Query::select()
            .column(crate::entity::refresh_tokens::Column::Id)
            .from(crate::entity::refresh_tokens::Entity)
            .and_where(crate::entity::refresh_tokens::Column::ExpiresAt.lt(before.naive_utc()))
            .limit(limit)
            .to_owned();
        let result = crate::entity::refresh_tokens::Entity::delete_many()
            .filter(crate::entity::refresh_tokens::Column::Id.in_subquery(oldest))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, JwtError> {
        let result = crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("revoked"),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(crate::entity::refresh_tokens::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::Database;
    use sqlx::Executor as _;

    use super::*;

    struct Seeded {
        repository: PostgresRefreshTokenRepository,
        pool: sqlx::PgPool,
        user_id: Uuid,
    }

    async fn seed() -> Seeded {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("refresh_rotation_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{schema}""#)))
            .await
            .expect("create test schema");

        let separator = if base_url.contains('?') { '&' } else { '?' };
        let schema_url = format!("{base_url}{separator}options=-c search_path={schema}");
        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        let db = Database::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        let realm_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, 'rotation', NOW(), NOW())")
            .bind(realm_id)
            .execute(&pool)
            .await
            .expect("seed a realm");
        sqlx::query("INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, 'u', '', '', 'u@example.com')")
            .bind(user_id)
            .bind(realm_id)
            .execute(&pool)
            .await
            .expect("seed a user");

        Seeded {
            repository: PostgresRefreshTokenRepository::new(db),
            pool,
            user_id,
        }
    }

    async fn issue(
        seeded: &Seeded,
        session_id: Option<Uuid>,
        resource: Option<&str>,
    ) -> RefreshToken {
        seeded
            .repository
            .create(
                Uuid::new_v4(),
                seeded.user_id,
                Some(Utc::now() + chrono::Duration::hours(1)),
                session_id,
                resource.map(str::to_string),
            )
            .await
            .expect("issue a refresh token")
    }

    async fn rotate(seeded: &Seeded, old: &RefreshToken) -> RotateOutcome {
        seeded
            .repository
            .rotate(
                old.id,
                Uuid::new_v4(),
                seeded.user_id,
                old.family_id,
                Some(Utc::now() + chrono::Duration::hours(1)),
            )
            .await
            .expect("rotate")
    }

    async fn rows_of_family(seeded: &Seeded, family_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE family_id = $1")
            .bind(family_id)
            .fetch_one(&seeded.pool)
            .await
            .expect("count the family")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_rotated_token_hands_its_session_and_resource_to_its_successor() {
        let seeded = seed().await;
        let session_id = Uuid::new_v4();
        let old = issue(&seeded, Some(session_id), Some("https://api.example")).await;

        let RotateOutcome::Rotated(successor) = rotate(&seeded, &old).await else {
            panic!("an active token must rotate");
        };

        assert_eq!(successor.family_id, old.family_id);
        assert_eq!(successor.user_id, seeded.user_id);
        assert_eq!(successor.status, RefreshTokenStatus::Active);
        assert_eq!(successor.resource.as_deref(), Some("https://api.example"));
        let successor_session: Option<Uuid> =
            sqlx::query_scalar("SELECT session_id FROM refresh_tokens WHERE id = $1")
                .bind(successor.id)
                .fetch_one(&seeded.pool)
                .await
                .expect("read the successor session");
        assert_eq!(successor_session, Some(session_id));

        let superseded = seeded
            .repository
            .get_by_jti(old.jti)
            .await
            .expect("read the superseded token");
        assert_eq!(superseded.status, RefreshTokenStatus::Rotated);
        assert_eq!(superseded.replaced_by, Some(successor.id));
        assert!(superseded.rotated_at.is_some());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_token_without_session_or_resource_rotates_into_one_without() {
        let seeded = seed().await;
        let old = issue(&seeded, None, None).await;

        let RotateOutcome::Rotated(successor) = rotate(&seeded, &old).await else {
            panic!("an active token must rotate");
        };

        assert_eq!(successor.resource, None);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_token_that_already_rotated_conflicts_and_mints_nothing() {
        let seeded = seed().await;
        let old = issue(&seeded, None, None).await;
        assert!(matches!(
            rotate(&seeded, &old).await,
            RotateOutcome::Rotated(_)
        ));

        let second = rotate(&seeded, &old).await;

        assert!(matches!(second, RotateOutcome::Conflict));
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_revoked_token_conflicts_and_mints_nothing() {
        let seeded = seed().await;
        let old = issue(&seeded, None, None).await;
        seeded
            .repository
            .revoke_by_jti(old.jti)
            .await
            .expect("revoke");

        let outcome = rotate(&seeded, &old).await;

        assert!(matches!(outcome, RotateOutcome::Conflict));
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn of_many_concurrent_rotations_exactly_one_wins() {
        let seeded = Arc::new(seed().await);
        let old = Arc::new(issue(&seeded, Some(Uuid::new_v4()), None).await);

        let attempts = (0..8).map(|_| {
            let seeded = Arc::clone(&seeded);
            let old = Arc::clone(&old);
            tokio::spawn(async move { rotate(&seeded, &old).await })
        });
        let mut won = 0;
        for attempt in attempts.collect::<Vec<_>>() {
            if matches!(attempt.await.expect("join"), RotateOutcome::Rotated(_)) {
                won += 1;
            }
        }

        assert_eq!(won, 1);
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 2);
    }
}
