use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use crate::domain::authentication::client_assertion::ports::ClientAssertionReplayStore;
use crate::domain::common::entities::app_errors::CoreError;
use crate::infrastructure::retention::{RETENTION_BATCH_SIZE, SWEEP_INTERVAL, purge_in_batches};

/// Keeps assertion `jti`s in Postgres so every replica sees the same set.
#[derive(Debug, Clone)]
pub struct PostgresClientAssertionReplayStore {
    db: DatabaseConnection,
}

impl PostgresClientAssertionReplayStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientAssertionReplayStore for PostgresClientAssertionReplayStore {
    async fn record(
        &self,
        client_id: Uuid,
        jti: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, CoreError> {
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO client_assertion_jtis (client_id, jti, expires_at)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (client_id, jti) DO UPDATE
                    SET expires_at = EXCLUDED.expires_at
                    WHERE client_assertion_jtis.expires_at < now()",
                [client_id.into(), jti.into(), expires_at.into()],
            ))
            .await
            .map_err(|error| {
                tracing::error!(%error, "could not record a client assertion jti");
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected() == 1)
    }

    async fn delete_expired(&self, before: DateTime<Utc>, limit: u64) -> Result<u64, CoreError> {
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM client_assertion_jtis
                 WHERE ctid IN (
                    SELECT ctid FROM client_assertion_jtis WHERE expires_at < $1 LIMIT $2
                 )",
                [before.into(), (limit as i64).into()],
            ))
            .await
            .map_err(|error| {
                tracing::error!(%error, "could not purge expired client assertion jtis");
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected())
    }
}

pub async fn client_assertion_jti_retention_task<S: ClientAssertionReplayStore>(store: S) {
    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        let purged = purge_in_batches("client_assertion_jtis", RETENTION_BATCH_SIZE, || {
            store.delete_expired(Utc::now(), RETENTION_BATCH_SIZE)
        })
        .await;

        if purged > 0 {
            tracing::info!(purged, "Retention: purged expired client assertion jtis");
        }
    }
}

#[cfg(test)]
mod db_tests {
    use chrono::{Duration, Utc};
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    use super::PostgresClientAssertionReplayStore;
    use crate::domain::authentication::client_assertion::ports::ClientAssertionReplayStore;

    async fn setup() -> (PostgresClientAssertionReplayStore, Uuid) {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("assertion_jti_test_{}", Uuid::new_v4().simple());

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
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("realm-{}", realm_id.simple()))
        .execute(&pool)
        .await
        .expect("insert realm");
        let client_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, \
             service_account_enabled, client_type, created_at, updated_at, \
             token_endpoint_auth_method) \
             VALUES ($1, $2, 'app', 'app', TRUE, 'openid-connect', FALSE, FALSE, 'confidential', \
             NOW(), NOW(), 'private_key_jwt')",
        )
        .bind(client_id)
        .bind(realm_id)
        .execute(&pool)
        .await
        .expect("insert client");

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");
        (PostgresClientAssertionReplayStore::new(db), client_id)
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_jti_is_accepted_once_until_it_expires() {
        let (store, client_id) = setup().await;
        let expires_at = Utc::now() + Duration::minutes(5);

        assert!(
            store
                .record(client_id, "jti-1", expires_at)
                .await
                .expect("record")
        );
        assert!(
            !store
                .record(client_id, "jti-1", expires_at)
                .await
                .expect("record")
        );
        assert!(
            store
                .record(client_id, "jti-2", expires_at)
                .await
                .expect("record")
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_expired_jti_can_be_used_again() {
        let (store, client_id) = setup().await;

        assert!(
            store
                .record(client_id, "jti", Utc::now() - Duration::seconds(1))
                .await
                .expect("record")
        );
        assert!(
            store
                .record(client_id, "jti", Utc::now() + Duration::minutes(5))
                .await
                .expect("record")
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn expired_jtis_are_purged() {
        let (store, client_id) = setup().await;
        store
            .record(client_id, "old", Utc::now() - Duration::minutes(1))
            .await
            .expect("record");
        store
            .record(client_id, "live", Utc::now() + Duration::minutes(5))
            .await
            .expect("record");

        assert_eq!(
            store.delete_expired(Utc::now(), 100).await.expect("purge"),
            1
        );
        assert!(
            !store
                .record(client_id, "live", Utc::now() + Duration::minutes(5))
                .await
                .expect("record")
        );
    }
}
