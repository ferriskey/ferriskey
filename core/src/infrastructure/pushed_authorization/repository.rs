use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use crate::domain::authentication::pushed_authorization::entities::{
    AuthorizationParameters, PushedAuthorizationRequest, RequestUriDigest,
};
use crate::domain::authentication::pushed_authorization::ports::PushedAuthorizationRequestRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::infrastructure::retention::{RETENTION_BATCH_SIZE, SWEEP_INTERVAL, purge_in_batches};

/// Keeps pushed requests in Postgres so `/auth` can land on any replica.
#[derive(Debug, Clone)]
pub struct PostgresPushedAuthorizationRequestRepository {
    db: DatabaseConnection,
}

impl PostgresPushedAuthorizationRequestRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn database_error(context: &'static str) -> impl FnOnce(sea_orm::DbErr) -> CoreError {
    move |error| {
        tracing::error!(%error, "{context}");
        CoreError::InternalServerError
    }
}

impl PushedAuthorizationRequestRepository for PostgresPushedAuthorizationRequestRepository {
    async fn store(&self, request: &PushedAuthorizationRequest) -> Result<(), CoreError> {
        let parameters = serde_json::to_value(&request.parameters).map_err(|error| {
            tracing::error!(%error, "could not encode pushed authorization parameters");
            CoreError::InternalServerError
        })?;
        let realm_id: Uuid = request.realm_id.into();

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO pushed_authorization_requests
                    (request_uri_hash, realm_id, client_id, parameters, expires_at)
                 VALUES ($1, $2, $3, $4, $5)",
                [
                    request.digest.as_str().into(),
                    realm_id.into(),
                    request.client_id.into(),
                    parameters.into(),
                    request.expires_at.into(),
                ],
            ))
            .await
            .map_err(database_error(
                "could not store a pushed authorization request",
            ))?;

        Ok(())
    }

    async fn take(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        digest: &RequestUriDigest,
        now: DateTime<Utc>,
    ) -> Result<Option<AuthorizationParameters>, CoreError> {
        let realm_id: Uuid = realm_id.into();
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM pushed_authorization_requests
                 WHERE request_uri_hash = $1
                   AND realm_id = $2
                   AND client_id = $3
                   AND expires_at > $4
                 RETURNING parameters",
                [
                    digest.as_str().into(),
                    realm_id.into(),
                    client_id.into(),
                    now.into(),
                ],
            ))
            .await
            .map_err(database_error(
                "could not take a pushed authorization request",
            ))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let parameters: serde_json::Value = row.try_get("", "parameters").map_err(
            database_error("could not read pushed authorization parameters"),
        )?;

        serde_json::from_value(parameters)
            .map(Some)
            .map_err(|error| {
                tracing::error!(%error, "could not decode pushed authorization parameters");
                CoreError::InternalServerError
            })
    }

    async fn delete_expired(&self, before: DateTime<Utc>, limit: u64) -> Result<u64, CoreError> {
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM pushed_authorization_requests
                 WHERE request_uri_hash IN (
                    SELECT request_uri_hash FROM pushed_authorization_requests
                    WHERE expires_at < $1 LIMIT $2
                 )",
                [before.into(), (limit as i64).into()],
            ))
            .await
            .map_err(database_error(
                "could not purge expired pushed authorization requests",
            ))?;

        Ok(result.rows_affected())
    }
}

pub async fn pushed_authorization_request_retention_task<
    S: PushedAuthorizationRequestRepository,
>(
    store: S,
) {
    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        let purged = purge_in_batches(
            "pushed_authorization_requests",
            RETENTION_BATCH_SIZE,
            || store.delete_expired(Utc::now(), RETENTION_BATCH_SIZE),
        )
        .await;

        if purged > 0 {
            tracing::info!(
                purged,
                "Retention: purged expired pushed authorization requests"
            );
        }
    }
}

#[cfg(test)]
mod db_tests {
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    use super::PostgresPushedAuthorizationRequestRepository;
    use crate::domain::authentication::pushed_authorization::services::tests::repository_contract;
    use crate::domain::realm::entities::RealmId;

    async fn setup() -> (
        PostgresPushedAuthorizationRequestRepository,
        RealmId,
        Uuid,
        Uuid,
    ) {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("par_test_{}", Uuid::new_v4().simple());

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

        let mut client_ids = Vec::with_capacity(2);
        for name in ["app", "other"] {
            let client_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, \
                 public_client, service_account_enabled, client_type, created_at, updated_at) \
                 VALUES ($1, $2, $3, $3, TRUE, 'openid-connect', TRUE, FALSE, 'public', \
                 NOW(), NOW())",
            )
            .bind(client_id)
            .bind(realm_id)
            .bind(name)
            .execute(&pool)
            .await
            .expect("insert client");
            client_ids.push(client_id);
        }

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");
        (
            PostgresPushedAuthorizationRequestRepository::new(db),
            RealmId::from(realm_id),
            client_ids[0],
            client_ids[1],
        )
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_postgres_store_honours_the_contract() {
        let (store, realm_id, client_id, other_client_id) = setup().await;

        repository_contract(&store, realm_id, client_id, other_client_id).await;
    }
}
