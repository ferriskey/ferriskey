use chrono::{DateTime, TimeZone, Utc};
use sea_orm::sea_query::{Expr, Query};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait,
    DatabaseConnection, DbBackend, EntityTrait, FromQueryResult, QueryFilter, Statement,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{
        JwtError, RefreshTokenHash,
        entities::{RefreshToken, RefreshTokenStatus},
        ports::{ConsumeOutcome, RefreshTokenRepository, RefreshTokenSuccessor, RotateOutcome},
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
            session_id: model.session_id,
            client_id: model.client_id,
            scope: model.scope,
        }
    }
}

const ROTATE_SQL: &str = "\
UPDATE refresh_tokens
SET previous_jti = jti, previous_token_hash = token_hash, jti = $3, token_hash = $4,
    expires_at = $5, created_at = $6, rotated_at = $7, replaced_by = NULL,
    client_id = $8, scope = $9
WHERE id = $1 AND jti = $2 AND status = 'active' AND user_id = $10 AND family_id = $11
RETURNING
        id, jti, user_id, revoked, expires_at, created_at, family_id, status, replaced_by,
        rotated_at, session_id, resource, token_hash, client_id, scope, previous_jti,
        previous_token_hash";

const CONSUME_AND_ROTATE_SQL: &str = "\
UPDATE refresh_tokens t
SET previous_jti = o.jti, previous_token_hash = o.token_hash, jti = $5, token_hash = $7,
    expires_at = $6, created_at = $4, rotated_at = $3, replaced_by = NULL
FROM (
    SELECT
            id, jti, user_id, revoked, expires_at, created_at, family_id, status, replaced_by,
            rotated_at, session_id, resource, token_hash, client_id, scope, previous_jti,
            previous_token_hash
    FROM refresh_tokens
    WHERE token_hash = $1 AND client_id = $2 AND status = 'active'
      AND (expires_at IS NULL OR expires_at > $4)
    FOR UPDATE
) o
WHERE t.id = o.id
RETURNING
    o.id AS c_id, o.jti AS c_jti, o.user_id AS c_user_id, o.revoked AS c_revoked,
    o.expires_at AS c_expires_at, o.created_at AS c_created_at, o.family_id AS c_family_id,
    o.status AS c_status, o.replaced_by AS c_replaced_by, o.rotated_at AS c_rotated_at,
    o.session_id AS c_session_id, o.resource AS c_resource, o.token_hash AS c_token_hash,
    o.client_id AS c_client_id, o.scope AS c_scope, o.previous_jti AS c_previous_jti,
    o.previous_token_hash AS c_previous_token_hash,
    t.id AS s_id, t.jti AS s_jti, t.user_id AS s_user_id, t.revoked AS s_revoked,
    t.expires_at AS s_expires_at, t.created_at AS s_created_at, t.family_id AS s_family_id,
    t.status AS s_status, t.replaced_by AS s_replaced_by, t.rotated_at AS s_rotated_at,
    t.session_id AS s_session_id, t.resource AS s_resource, t.token_hash AS s_token_hash,
    t.client_id AS s_client_id, t.scope AS s_scope, t.previous_jti AS s_previous_jti,
    t.previous_token_hash AS s_previous_token_hash";

/// A row holds its current token and the one before it. Presenting the previous one reads as the
/// superseded token it is, whatever the row's own status says about the current one.
fn as_presented(mut model: crate::entity::refresh_tokens::Model, current: bool) -> RefreshToken {
    if !current {
        if model.status != "revoked" {
            model.status = "rotated".to_string();
        }
        if let Some(previous_jti) = model.previous_jti {
            model.jti = previous_jti;
        }
        model.token_hash = model.previous_token_hash.take();
    }
    model.into()
}

fn hash_value(hash: Option<RefreshTokenHash>) -> Option<Vec<u8>> {
    hash.map(|hash| hash.0.to_vec())
}

#[derive(Debug, Clone)]
pub struct PostgresRefreshTokenRepository {
    pub db: DatabaseConnection,
    issued_db: DatabaseConnection,
}

impl PostgresRefreshTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            issued_db: db.clone(),
            db,
        }
    }

    /// Issued and rotated tokens are written through `issued_db`. Revocations, lookups and
    /// cleanup keep `db`, so a revocation is never lost on a database crash.
    pub fn with_issued_tokens_written_through(mut self, issued_db: DatabaseConnection) -> Self {
        self.issued_db = issued_db;
        self
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
        token_hash: Option<RefreshTokenHash>,
        client_id: Option<String>,
        scope: Option<String>,
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
            token_hash: Set(hash_value(token_hash)),
            client_id: Set(client_id),
            scope: Set(scope),
            previous_jti: Set(None),
            previous_token_hash: Set(None),
        };

        let refresh_token = model
            .insert(&self.issued_db)
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
        token_hash: Option<RefreshTokenHash>,
        client_id: Option<String>,
        scope: Option<String>,
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
            token_hash: Set(hash_value(token_hash)),
            client_id: Set(client_id),
            scope: Set(scope),
            previous_jti: Set(None),
            previous_token_hash: Set(None),
        };

        let refresh_token = model
            .insert(&self.issued_db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(refresh_token.into())
    }

    async fn get_by_jti(&self, jti: Uuid) -> Result<RefreshToken, JwtError> {
        let model = crate::entity::refresh_tokens::Entity::find()
            .filter(
                Condition::any()
                    .add(crate::entity::refresh_tokens::Column::Jti.eq(jti))
                    .add(crate::entity::refresh_tokens::Column::PreviousJti.eq(jti)),
            )
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?
            .ok_or(JwtError::InvalidToken)?;

        let current = model.jti == jti;
        Ok(as_presented(model, current))
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
        old_jti: Uuid,
        new_jti: Uuid,
        user_id: Uuid,
        family_id: Uuid,
        new_expires_at: Option<DateTime<Utc>>,
        token_hash: Option<RefreshTokenHash>,
        client_id: Option<String>,
        scope: Option<String>,
    ) -> Result<RotateOutcome, JwtError> {
        let now = Utc::now();

        let row = self
            .issued_db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                ROTATE_SQL,
                [
                    old_id.into(),
                    old_jti.into(),
                    new_jti.into(),
                    hash_value(token_hash).into(),
                    new_expires_at.map(|dt| dt.naive_utc()).into(),
                    now.naive_utc().into(),
                    now.fixed_offset().into(),
                    client_id.into(),
                    scope.into(),
                    user_id.into(),
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

    async fn consume_and_rotate(
        &self,
        hash: RefreshTokenHash,
        client_id: &str,
        successor: RefreshTokenSuccessor,
    ) -> Result<ConsumeOutcome, JwtError> {
        let now = Utc::now();

        let row = self
            .issued_db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                CONSUME_AND_ROTATE_SQL,
                [
                    hash.0.to_vec().into(),
                    client_id.into(),
                    now.fixed_offset().into(),
                    now.naive_utc().into(),
                    successor.jti.into(),
                    successor.expires_at.map(|dt| dt.naive_utc()).into(),
                    hash_value(successor.token_hash).into(),
                ],
            ))
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(ConsumeOutcome::NotConsumed);
        };

        let consumed = crate::entity::refresh_tokens::Model::from_query_result(&row, "c_")
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;
        let successor = crate::entity::refresh_tokens::Model::from_query_result(&row, "s_")
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(ConsumeOutcome::Rotated {
            consumed: consumed.into(),
            successor: successor.into(),
        })
    }

    async fn get_by_hash(&self, hash: RefreshTokenHash) -> Result<Option<RefreshToken>, JwtError> {
        let model = crate::entity::refresh_tokens::Entity::find()
            .filter(
                Condition::any()
                    .add(crate::entity::refresh_tokens::Column::TokenHash.eq(hash.0.to_vec()))
                    .add(
                        crate::entity::refresh_tokens::Column::PreviousTokenHash
                            .eq(hash.0.to_vec()),
                    ),
            )
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(model.map(|model| {
            let current = model.token_hash.as_deref() == Some(hash.0.as_slice());
            as_presented(model, current)
        }))
    }

    async fn revoke_by_hash(
        &self,
        hash: RefreshTokenHash,
        client_id: &str,
        realm_id: Uuid,
    ) -> Result<bool, JwtError> {
        let realm_users = Query::select()
            .column(crate::entity::users::Column::Id)
            .from(crate::entity::users::Entity)
            .and_where(crate::entity::users::Column::RealmId.eq(realm_id))
            .to_owned();
        let result = crate::entity::refresh_tokens::Entity::update_many()
            .col_expr(
                crate::entity::refresh_tokens::Column::Revoked,
                Expr::value(true),
            )
            .col_expr(
                crate::entity::refresh_tokens::Column::Status,
                Expr::value("revoked"),
            )
            .filter(crate::entity::refresh_tokens::Column::TokenHash.eq(hash.0.to_vec()))
            .filter(crate::entity::refresh_tokens::Column::ClientId.eq(client_id))
            .filter(crate::entity::refresh_tokens::Column::UserId.in_subquery(realm_users))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected > 0)
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

    use ferriskey_security::jwt::OpaqueRefreshToken;
    use sea_orm::Database;
    use sqlx::Executor as _;

    use super::*;

    struct Seeded {
        repository: PostgresRefreshTokenRepository,
        pool: sqlx::PgPool,
        user_id: Uuid,
        realm_id: Uuid,
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
            realm_id,
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
                None,
                None,
                None,
            )
            .await
            .expect("issue a refresh token")
    }

    async fn issue_opaque(
        seeded: &Seeded,
        client_id: &str,
        lifetime: chrono::Duration,
    ) -> (OpaqueRefreshToken, RefreshToken) {
        let token = OpaqueRefreshToken::generate();
        let stored = seeded
            .repository
            .create(
                Uuid::new_v4(),
                seeded.user_id,
                Some(Utc::now() + lifetime),
                Some(Uuid::new_v4()),
                Some("https://api.example".to_string()),
                Some(token.hash()),
                Some(client_id.to_string()),
                Some("openid profile".to_string()),
            )
            .await
            .expect("issue an opaque refresh token");
        (token, stored)
    }

    fn successor() -> RefreshTokenSuccessor {
        RefreshTokenSuccessor {
            jti: Uuid::new_v4(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_hash: Some(OpaqueRefreshToken::generate().hash()),
        }
    }

    async fn consume(
        seeded: &Seeded,
        token: &OpaqueRefreshToken,
        client_id: &str,
    ) -> ConsumeOutcome {
        seeded
            .repository
            .consume_and_rotate(token.hash(), client_id, successor())
            .await
            .expect("consume")
    }

    async fn rotate(seeded: &Seeded, old: &RefreshToken) -> RotateOutcome {
        seeded
            .repository
            .rotate(
                old.id,
                old.jti,
                Uuid::new_v4(),
                seeded.user_id,
                old.family_id,
                Some(Utc::now() + chrono::Duration::hours(1)),
                None,
                None,
                None,
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

        assert_eq!(successor.id, old.id);
        assert_eq!(successor.family_id, old.family_id);
        assert_eq!(successor.user_id, seeded.user_id);
        assert_eq!(successor.status, RefreshTokenStatus::Active);
        assert_eq!(successor.resource.as_deref(), Some("https://api.example"));
        assert_eq!(successor.session_id, Some(session_id));
        assert_ne!(successor.jti, old.jti);
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 1);

        let superseded = seeded
            .repository
            .get_by_jti(old.jti)
            .await
            .expect("read the superseded token");
        assert_eq!(superseded.status, RefreshTokenStatus::Rotated);
        assert_eq!(superseded.jti, old.jti);
        assert_eq!(superseded.id, old.id);
        assert!(successor.rotated_at.is_some());
        let previous: Option<Uuid> =
            sqlx::query_scalar("SELECT previous_jti FROM refresh_tokens WHERE id = $1")
                .bind(old.id)
                .fetch_one(&seeded.pool)
                .await
                .expect("read the previous jti");
        assert_eq!(previous, Some(old.jti));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_token_tables_keep_no_index_on_a_flag_or_a_status() {
        let seeded = seed().await;

        let indexes: Vec<String> = sqlx::query_scalar(
            "SELECT indexname FROM pg_indexes \
             WHERE schemaname = current_schema() AND tablename IN ('access_tokens', 'refresh_tokens') \
             ORDER BY indexname",
        )
        .fetch_all(&seeded.pool)
        .await
        .expect("list the token indexes");

        assert!(
            !indexes
                .iter()
                .any(|name| name == "idx_access_tokens_revoked"),
            "{indexes:?}"
        );
        assert!(
            !indexes
                .iter()
                .any(|name| name == "idx_refresh_tokens_status"),
            "{indexes:?}"
        );
        for kept in [
            "idx_access_tokens_claims_sid",
            "idx_access_tokens_expires_at",
            "idx_refresh_tokens_family_id",
            "idx_refresh_tokens_previous_jti",
            "idx_refresh_tokens_previous_token_hash",
            "idx_refresh_tokens_session_id",
        ] {
            assert!(
                indexes.iter().any(|name| name == kept),
                "{kept} in {indexes:?}"
            );
        }
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
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 1);
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
        assert_eq!(rows_of_family(&seeded, old.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn consuming_an_opaque_token_hands_everything_but_the_secret_to_its_successor() {
        let seeded = seed().await;
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;

        let ConsumeOutcome::Rotated {
            consumed,
            successor,
        } = consume(&seeded, &token, "web").await
        else {
            panic!("an active token of its own client must be consumed");
        };

        assert_eq!(consumed.id, stored.id);
        assert_eq!(consumed.jti, stored.jti);
        assert_eq!(successor.id, stored.id);
        assert_ne!(successor.jti, stored.jti);
        assert_eq!(consumed.session_id, stored.session_id);
        assert_eq!(consumed.scope.as_deref(), Some("openid profile"));
        assert_eq!(successor.status, RefreshTokenStatus::Active);
        assert_eq!(successor.user_id, stored.user_id);
        assert_eq!(successor.family_id, stored.family_id);
        assert_eq!(successor.session_id, stored.session_id);
        assert_eq!(successor.resource.as_deref(), Some("https://api.example"));
        assert_eq!(successor.client_id.as_deref(), Some("web"));
        assert_eq!(successor.scope.as_deref(), Some("openid profile"));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_successor_is_stored_under_its_own_hash() {
        let seeded = seed().await;
        let (token, _) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        let next = successor();
        let next_hash = next.token_hash.expect("an opaque successor");

        seeded
            .repository
            .consume_and_rotate(token.hash(), "web", next)
            .await
            .expect("consume");

        let found = seeded
            .repository
            .get_by_hash(next_hash)
            .await
            .expect("lookup")
            .expect("the successor is stored under its hash");
        assert_eq!(found.status, RefreshTokenStatus::Active);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_token_presented_by_another_client_is_not_consumed() {
        let seeded = seed().await;
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;

        let outcome = consume(&seeded, &token, "other").await;

        assert!(matches!(outcome, ConsumeOutcome::NotConsumed));
        let after = seeded
            .repository
            .get_by_hash(token.hash())
            .await
            .expect("lookup")
            .expect("still stored");
        assert_eq!(after.status, RefreshTokenStatus::Active);
        assert_eq!(rows_of_family(&seeded, stored.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_expired_token_is_not_consumed() {
        let seeded = seed().await;
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::seconds(-5)).await;

        let outcome = consume(&seeded, &token, "web").await;

        assert!(matches!(outcome, ConsumeOutcome::NotConsumed));
        assert_eq!(rows_of_family(&seeded, stored.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_unknown_token_is_not_consumed() {
        let seeded = seed().await;

        let outcome = consume(&seeded, &OpaqueRefreshToken::generate(), "web").await;

        assert!(matches!(outcome, ConsumeOutcome::NotConsumed));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_consumed_token_is_not_consumed_twice() {
        let seeded = seed().await;
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        assert!(matches!(
            consume(&seeded, &token, "web").await,
            ConsumeOutcome::Rotated { .. }
        ));

        let second = consume(&seeded, &token, "web").await;

        assert!(matches!(second, ConsumeOutcome::NotConsumed));
        assert_eq!(rows_of_family(&seeded, stored.family_id).await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn of_many_concurrent_consumptions_exactly_one_wins() {
        let seeded = Arc::new(seed().await);
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        let token = Arc::new(token);

        let attempts = (0..8).map(|_| {
            let seeded = Arc::clone(&seeded);
            let token = Arc::clone(&token);
            tokio::spawn(async move { consume(&seeded, &token, "web").await })
        });
        let mut won = 0;
        for attempt in attempts.collect::<Vec<_>>() {
            if matches!(attempt.await.expect("join"), ConsumeOutcome::Rotated { .. }) {
                won += 1;
            }
        }

        assert_eq!(won, 1);
        assert_eq!(rows_of_family(&seeded, stored.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_by_hash_only_touches_the_token_of_that_client() {
        let seeded = Arc::new(seed().await);
        let (token, _) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;

        let by_other = seeded
            .repository
            .revoke_by_hash(token.hash(), "other", seeded.realm_id)
            .await
            .expect("revoke");
        let still_active = seeded
            .repository
            .get_by_hash(token.hash())
            .await
            .expect("lookup")
            .expect("stored")
            .status;
        let by_other_realm = seeded
            .repository
            .revoke_by_hash(token.hash(), "web", Uuid::new_v4())
            .await
            .expect("revoke");
        let by_owner = seeded
            .repository
            .revoke_by_hash(token.hash(), "web", seeded.realm_id)
            .await
            .expect("revoke");
        let revoked = seeded
            .repository
            .get_by_hash(token.hash())
            .await
            .expect("lookup")
            .expect("stored")
            .status;

        assert!(!by_other);
        assert!(!by_other_realm);
        assert_eq!(still_active, RefreshTokenStatus::Active);
        assert!(by_owner);
        assert_eq!(revoked, RefreshTokenStatus::Revoked);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_legacy_rotation_can_hand_an_opaque_successor_its_hash() {
        let seeded = seed().await;
        let old = issue(&seeded, None, None).await;
        let next = OpaqueRefreshToken::generate();

        let outcome = seeded
            .repository
            .rotate(
                old.id,
                old.jti,
                Uuid::new_v4(),
                seeded.user_id,
                old.family_id,
                Some(Utc::now() + chrono::Duration::hours(1)),
                Some(next.hash()),
                Some("web".to_string()),
                Some("openid".to_string()),
            )
            .await
            .expect("rotate");

        assert!(matches!(outcome, RotateOutcome::Rotated(_)));
        let found = seeded
            .repository
            .get_by_hash(next.hash())
            .await
            .expect("lookup")
            .expect("stored under its hash");
        assert_eq!(found.client_id.as_deref(), Some("web"));
        assert_eq!(found.scope.as_deref(), Some("openid"));
    }

    async fn current_hash(seeded: &Seeded, id: Uuid) -> RefreshTokenHash {
        let hash: Vec<u8> =
            sqlx::query_scalar("SELECT token_hash FROM refresh_tokens WHERE id = $1")
                .bind(id)
                .fetch_one(&seeded.pool)
                .await
                .expect("read the current hash");
        RefreshTokenHash(hash.try_into().expect("a sha256 hash"))
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_previous_opaque_token_reads_as_rotated_and_an_older_one_as_unknown() {
        let seeded = seed().await;
        let (first, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        assert!(matches!(
            consume(&seeded, &first, "web").await,
            ConsumeOutcome::Rotated { .. }
        ));
        let found = seeded
            .repository
            .get_by_hash(first.hash())
            .await
            .expect("lookup")
            .expect("the previous token is still known");
        assert_eq!(found.status, RefreshTokenStatus::Rotated);
        assert_eq!(found.family_id, stored.family_id);
        assert_eq!(found.jti, stored.jti);

        let second = current_hash(&seeded, stored.id).await;
        let third = OpaqueRefreshToken::generate();
        let outcome = seeded
            .repository
            .consume_and_rotate(
                second,
                "web",
                RefreshTokenSuccessor {
                    jti: Uuid::new_v4(),
                    expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
                    token_hash: Some(third.hash()),
                },
            )
            .await
            .expect("consume");
        assert!(matches!(outcome, ConsumeOutcome::Rotated { .. }));

        assert!(
            seeded
                .repository
                .get_by_hash(first.hash())
                .await
                .expect("lookup")
                .is_none()
        );
        assert_eq!(
            seeded
                .repository
                .get_by_hash(second)
                .await
                .expect("lookup")
                .expect("the new previous token is known")
                .status,
            RefreshTokenStatus::Rotated
        );
        assert_eq!(
            seeded
                .repository
                .get_by_hash(third.hash())
                .await
                .expect("lookup")
                .expect("the current token is known")
                .status,
            RefreshTokenStatus::Active
        );
        assert_eq!(rows_of_family(&seeded, stored.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_previous_legacy_jti_reads_as_rotated_and_an_older_one_as_unknown() {
        let seeded = seed().await;
        let first = issue(&seeded, None, None).await;
        let RotateOutcome::Rotated(second) = rotate(&seeded, &first).await else {
            panic!("an active token must rotate");
        };
        assert_eq!(
            seeded
                .repository
                .get_by_jti(first.jti)
                .await
                .expect("the previous jti is known")
                .status,
            RefreshTokenStatus::Rotated
        );
        assert_eq!(
            seeded
                .repository
                .get_by_jti(second.jti)
                .await
                .expect("the current jti is known")
                .status,
            RefreshTokenStatus::Active
        );

        assert!(matches!(
            rotate(&seeded, &second).await,
            RotateOutcome::Rotated(_)
        ));

        assert!(seeded.repository.get_by_jti(first.jti).await.is_err());
        assert_eq!(
            seeded
                .repository
                .get_by_jti(second.jti)
                .await
                .expect("the new previous jti is known")
                .status,
            RefreshTokenStatus::Rotated
        );
        assert!(matches!(
            rotate(&seeded, &first).await,
            RotateOutcome::Conflict
        ));
        assert_eq!(rows_of_family(&seeded, first.family_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_previous_token_is_revoked_with_its_family_but_not_by_its_own_hash() {
        let seeded = seed().await;
        let (first, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        assert!(matches!(
            consume(&seeded, &first, "web").await,
            ConsumeOutcome::Rotated { .. }
        ));

        let revoked = seeded
            .repository
            .revoke_by_hash(first.hash(), "web", seeded.realm_id)
            .await
            .expect("revoke");
        assert!(!revoked);
        assert_eq!(
            seeded
                .repository
                .get_by_hash(first.hash())
                .await
                .expect("lookup")
                .expect("known")
                .status,
            RefreshTokenStatus::Rotated
        );

        seeded
            .repository
            .revoke_family(stored.family_id)
            .await
            .expect("revoke the family");
        assert_eq!(
            seeded
                .repository
                .get_by_hash(first.hash())
                .await
                .expect("lookup")
                .expect("known")
                .status,
            RefreshTokenStatus::Revoked
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn rows_rotated_by_the_former_model_still_report_rotated() {
        let seeded = seed().await;
        let (token, stored) = issue_opaque(&seeded, "web", chrono::Duration::hours(1)).await;
        sqlx::query(
            "UPDATE refresh_tokens SET status = 'rotated', replaced_by = $2, rotated_at = NOW() \
             WHERE id = $1",
        )
        .bind(stored.id)
        .bind(Uuid::new_v4())
        .execute(&seeded.pool)
        .await
        .expect("age the row into the former model");

        let by_hash = seeded
            .repository
            .get_by_hash(token.hash())
            .await
            .expect("lookup")
            .expect("known");
        let by_jti = seeded
            .repository
            .get_by_jti(stored.jti)
            .await
            .expect("known");

        assert_eq!(by_hash.status, RefreshTokenStatus::Rotated);
        assert_eq!(by_jti.status, RefreshTokenStatus::Rotated);
        assert!(matches!(
            consume(&seeded, &token, "web").await,
            ConsumeOutcome::NotConsumed
        ));
    }
}
