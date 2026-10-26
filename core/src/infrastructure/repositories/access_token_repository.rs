use std::sync::{Arc, OnceLock};

use chrono::{DateTime, TimeZone, Utc};
use sea_orm::sea_query::{Expr, OnConflict, Query};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Statement,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{JwtError, entities::AccessToken, ports::AccessTokenRepository},
    realm::entities::Unscoped,
};
use crate::entity::access_tokens;
use crate::infrastructure::repositories::access_token_writer::{CoalescingWriter, RowSink};

impl From<crate::entity::access_tokens::Model> for AccessToken {
    fn from(model: crate::entity::access_tokens::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = model.expires_at.map(|dt| Utc.from_utc_datetime(&dt));

        AccessToken::new(
            model.id,
            model.token_hash,
            model.jti,
            model.user_id,
            model.realm_id,
            model.revoked,
            expires_at,
            model.claims,
            created_at,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PostgresAccessTokenRepository {
    pub db: DatabaseConnection,
    writer: Arc<OnceLock<CoalescingWriter<access_tokens::ActiveModel>>>,
}

impl PostgresAccessTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            writer: Arc::new(OnceLock::new()),
        }
    }

    fn writer(&self) -> &CoalescingWriter<access_tokens::ActiveModel> {
        self.writer.get_or_init(|| {
            CoalescingWriter::spawn(Arc::new(AccessTokenSink {
                db: self.db.clone(),
            }))
        })
    }
}

struct AccessTokenSink {
    db: DatabaseConnection,
}

impl RowSink for AccessTokenSink {
    type Row = access_tokens::ActiveModel;

    async fn insert_many(&self, rows: &[Self::Row]) -> Result<(), String> {
        access_tokens::Entity::insert_many(rows.iter().cloned())
            .exec_without_returning(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn insert_one(&self, row: &Self::Row) -> Result<(), String> {
        access_tokens::Entity::insert(row.clone())
            .exec_without_returning(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

impl AccessTokenRepository for PostgresAccessTokenRepository {
    async fn create(
        &self,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: ferriskey_domain::realm::RealmId,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
    ) -> Result<AccessToken, JwtError> {
        let id = generate_uuid_v7();
        let created_at = Utc::now();
        let realm_id = Uuid::from(realm_id);
        let access_token = AccessToken::new(
            id,
            token_hash.clone(),
            jti,
            user_id,
            realm_id,
            false,
            expires_at,
            claims.clone(),
            created_at,
        );

        let row = access_tokens::ActiveModel {
            id: Set(id),
            token_hash: Set(token_hash),
            jti: Set(jti),
            user_id: Set(user_id),
            realm_id: Set(realm_id),
            revoked: Set(false),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
            claims: Set(claims),
            created_at: Set(created_at.naive_utc()),
        };

        self.writer()
            .write(row)
            .await
            .map_err(JwtError::GenerationError)?;

        Ok(access_token)
    }

    async fn get_by_token_hash(
        &self,
        token_hash: String,
    ) -> Result<Option<Unscoped<AccessToken>>, JwtError> {
        let access_token = crate::entity::access_tokens::Entity::find()
            .filter(crate::entity::access_tokens::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(access_token.map(|t| Unscoped::new(t.into())))
    }

    async fn revoke_by_session_id(&self, session_id: Uuid) -> Result<u64, JwtError> {
        let result = crate::entity::access_tokens::Entity::update_many()
            .col_expr(
                crate::entity::access_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(Expr::cust_with_values(
                "claims ->> 'sid' = $1",
                [session_id.to_string()],
            ))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn delete_expired(&self, before: DateTime<Utc>, limit: u64) -> Result<u64, JwtError> {
        let oldest = Query::select()
            .column(crate::entity::access_tokens::Column::Id)
            .from(crate::entity::access_tokens::Entity)
            .and_where(crate::entity::access_tokens::Column::ExpiresAt.lt(before.naive_utc()))
            .limit(limit)
            .to_owned();
        let result = crate::entity::access_tokens::Entity::delete_many()
            .filter(crate::entity::access_tokens::Column::Id.in_subquery(oldest))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, JwtError> {
        let result = crate::entity::access_tokens::Entity::update_many()
            .col_expr(
                crate::entity::access_tokens::Column::Revoked,
                Expr::value(true),
            )
            .filter(crate::entity::access_tokens::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        // Tokens that were never stored (client credentials) are caught by date instead.
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO user_token_revocations (user_id, revoked_before) \
                 VALUES ($1, date_trunc('second', now() AT TIME ZONE 'UTC')) \
                 ON CONFLICT (user_id) DO UPDATE SET revoked_before = EXCLUDED.revoked_before",
                [user_id.into()],
            ))
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn is_revoked(
        &self,
        token_hash: String,
        user_id: Uuid,
        issued_at: i64,
    ) -> Result<bool, JwtError> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT EXISTS (SELECT 1 FROM access_tokens WHERE token_hash = $1 AND revoked) \
                     OR EXISTS (SELECT 1 FROM user_token_revocations \
                                WHERE user_id = $2 \
                                  AND revoked_before > to_timestamp($3) AT TIME ZONE 'UTC') \
                     AS revoked",
                [token_hash.into(), user_id.into(), issued_at.into()],
            ))
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        row.map_or(Ok(false), |row| {
            row.try_get::<bool>("", "revoked")
                .map_err(|e| JwtError::GenerationError(e.to_string()))
        })
    }

    async fn revoke_by_token_hash(
        &self,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: ferriskey_domain::realm::RealmId,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
    ) -> Result<(), JwtError> {
        let row = access_tokens::ActiveModel {
            id: Set(generate_uuid_v7()),
            token_hash: Set(token_hash),
            jti: Set(jti),
            user_id: Set(user_id),
            realm_id: Set(Uuid::from(realm_id)),
            revoked: Set(true),
            expires_at: Set(expires_at.map(|dt| dt.naive_utc())),
            claims: Set(claims),
            created_at: Set(Utc::now().naive_utc()),
        };

        access_tokens::Entity::insert(row)
            .on_conflict(
                OnConflict::column(access_tokens::Column::TokenHash)
                    .update_column(access_tokens::Column::Revoked)
                    .to_owned(),
            )
            .exec_without_returning(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    struct Fixture {
        repository: PostgresAccessTokenRepository,
        realm_id: Uuid,
        user_id: Uuid,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("access_token_repository_test_{}", Uuid::new_v4().simple());

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

        let realm_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, 'perf', NOW(), NOW())",
        )
        .bind(realm_id)
        .execute(&pool)
        .await
        .expect("insert realm");

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, enabled, email_verified, created_at, updated_at) \
             VALUES ($1, $2, 'service-account', true, false, NOW(), NOW())",
        )
        .bind(user_id)
        .bind(realm_id)
        .execute(&pool)
        .await
        .expect("insert user");

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        Fixture {
            repository: PostgresAccessTokenRepository::new(db),
            realm_id,
            user_id,
        }
    }

    async fn create(fixture: &Fixture, token_hash: &str) -> Result<AccessToken, JwtError> {
        fixture
            .repository
            .create(
                token_hash.to_string(),
                Some(Uuid::new_v4()),
                fixture.user_id,
                fixture.realm_id.into(),
                None,
                serde_json::json!({ "sub": fixture.user_id }),
            )
            .await
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_created_token_can_be_found_by_its_hash() {
        let fixture = setup().await;

        let created = create(&fixture, "hash-1").await.expect("create");

        let found = fixture
            .repository
            .get_by_token_hash("hash-1".to_string())
            .await
            .expect("lookup")
            .expect("token stored")
            .across_realms();
        assert_eq!(found.id, created.id);
        assert!(!found.revoked);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn tokens_created_at_the_same_time_are_all_stored() {
        let fixture = Arc::new(setup().await);
        let mut creations = tokio::task::JoinSet::new();

        for index in 0..200 {
            let fixture = Arc::clone(&fixture);
            creations.spawn(async move { create(&fixture, &format!("hash-{index}")).await });
        }
        while let Some(done) = creations.join_next().await {
            done.expect("task").expect("create");
        }

        for index in 0..200 {
            let found = fixture
                .repository
                .get_by_token_hash(format!("hash-{index}"))
                .await
                .expect("lookup");
            assert!(found.is_some(), "hash-{index} was not stored");
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_duplicate_hash_fails_alone_and_does_not_lose_the_others() {
        let fixture = Arc::new(setup().await);
        create(&fixture, "taken").await.expect("first create");
        let mut creations = tokio::task::JoinSet::new();

        for hash in ["taken", "fresh-1", "fresh-2", "fresh-3"] {
            let fixture = Arc::clone(&fixture);
            creations.spawn(async move { (hash, create(&fixture, hash).await.is_ok()) });
        }
        let mut outcomes = Vec::new();
        while let Some(done) = creations.join_next().await {
            outcomes.push(done.expect("task"));
        }
        outcomes.sort_unstable();

        assert_eq!(
            outcomes,
            vec![
                ("fresh-1", true),
                ("fresh-2", true),
                ("fresh-3", true),
                ("taken", false)
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_token_that_was_never_stored_is_caught_when_all_the_users_tokens_are_revoked() {
        let fixture = setup().await;
        let now = Utc::now().timestamp();

        let before = fixture
            .repository
            .is_revoked("unstored".to_string(), fixture.user_id, now - 5)
            .await
            .expect("lookup");
        assert!(!before);

        fixture
            .repository
            .revoke_all_for_user(fixture.user_id)
            .await
            .expect("revoke all");

        let older = fixture
            .repository
            .is_revoked("unstored".to_string(), fixture.user_id, now - 5)
            .await
            .expect("lookup");
        let newer = fixture
            .repository
            .is_revoked("unstored".to_string(), fixture.user_id, now + 5)
            .await
            .expect("lookup");
        assert!(older, "a token issued before the revocation is refused");
        assert!(!newer, "a token issued after the revocation stays valid");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_a_token_that_was_never_stored_refuses_that_token_only() {
        let fixture = setup().await;
        let now = Utc::now().timestamp();

        fixture
            .repository
            .revoke_by_token_hash(
                "unstored".to_string(),
                Some(Uuid::new_v4()),
                fixture.user_id,
                fixture.realm_id.into(),
                None,
                serde_json::json!({ "sub": fixture.user_id }),
            )
            .await
            .expect("revoke");

        let revoked = fixture
            .repository
            .is_revoked("unstored".to_string(), fixture.user_id, now)
            .await
            .expect("lookup");
        let other = fixture
            .repository
            .is_revoked("another".to_string(), fixture.user_id, now)
            .await
            .expect("lookup");
        assert!(revoked);
        assert!(!other);
    }
}
