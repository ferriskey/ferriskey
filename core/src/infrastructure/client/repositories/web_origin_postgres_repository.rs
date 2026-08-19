use std::collections::HashMap;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    SqlErr,
};
use uuid::Uuid;

use crate::domain::client::{
    entities::web_origin::{WebOrigin, WebOriginValue},
    ports::WebOriginRepository,
    web_origin_resolution::ClientOriginSources,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{
    client_web_origins::{ActiveModel, Column as WebOriginColumn, Entity as WebOriginEntity},
    clients::{Column as ClientColumn, Entity as ClientEntity},
    realms::{Column as RealmColumn, Entity as RealmEntity},
    redirect_uris::{Column as RedirectUriColumn, Entity as RedirectUriEntity},
};

#[derive(Debug, Clone)]
pub struct PostgresWebOriginRepository {
    pub db: DatabaseConnection,
}

impl PostgresWebOriginRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn enabled_client_ids_in_realm(&self, realm_name: &str) -> Result<Vec<Uuid>, CoreError> {
        let realm = RealmEntity::find()
            .filter(RealmColumn::Name.eq(realm_name))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let Some(realm) = realm else {
            return Ok(Vec::new());
        };

        let clients = ClientEntity::find()
            .filter(ClientColumn::RealmId.eq(realm.id))
            .filter(ClientColumn::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(clients.into_iter().map(|client| client.id).collect())
    }

    async fn enabled_redirect_uris_for(
        &self,
        client_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<String>>, CoreError> {
        if client_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = RedirectUriEntity::find()
            .filter(RedirectUriColumn::ClientId.is_in(client_ids))
            .filter(RedirectUriColumn::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut by_client: HashMap<Uuid, Vec<String>> = HashMap::new();
        for row in rows {
            by_client.entry(row.client_id).or_default().push(row.value);
        }

        Ok(by_client)
    }
}

impl WebOriginRepository for PostgresWebOriginRepository {
    async fn create(&self, client_id: Uuid, value: WebOriginValue) -> Result<WebOrigin, CoreError> {
        let web_origin = WebOrigin::new(client_id, value);

        let payload = ActiveModel {
            id: Set(web_origin.id),
            client_id: Set(web_origin.client_id),
            value: Set(web_origin.value.to_string()),
            created_at: Set(web_origin.created_at.naive_utc()),
            updated_at: Set(web_origin.updated_at.naive_utc()),
        };

        let inserted = payload
            .insert(&self.db)
            .await
            .map_err(|error| match error.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => CoreError::InvalidWebOrigin(
                    "this origin is already registered for the client".to_string(),
                ),
                _ => CoreError::InternalServerError,
            })?;

        WebOrigin::try_from(inserted)
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<WebOrigin>, CoreError> {
        WebOriginEntity::find()
            .filter(WebOriginColumn::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into_iter()
            .map(WebOrigin::try_from)
            .collect()
    }

    async fn delete(&self, client_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = WebOriginEntity::delete_many()
            .filter(WebOriginColumn::Id.eq(id))
            .filter(WebOriginColumn::ClientId.eq(client_id))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if result.rows_affected == 0 {
            return Err(CoreError::WebOriginNotFound);
        }

        Ok(())
    }

    async fn get_origin_sources_by_realm_name(
        &self,
        realm_name: String,
    ) -> Result<Vec<ClientOriginSources>, CoreError> {
        let client_ids = self.enabled_client_ids_in_realm(&realm_name).await?;

        if client_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = WebOriginEntity::find()
            .filter(WebOriginColumn::ClientId.is_in(client_ids))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut by_client: HashMap<Uuid, Vec<WebOriginValue>> = HashMap::new();
        for row in rows {
            let client_id = row.client_id;
            let value = WebOrigin::try_from(row).map(|origin| origin.value)?;
            by_client.entry(client_id).or_default().push(value);
        }

        let derived: Vec<Uuid> = by_client
            .iter()
            .filter(|(_, values)| values.contains(&WebOriginValue::DerivedFromRedirectUris))
            .map(|(client_id, _)| *client_id)
            .collect();

        let mut redirect_uris = self.enabled_redirect_uris_for(derived).await?;

        Ok(by_client
            .into_iter()
            .map(|(client_id, web_origins)| ClientOriginSources {
                web_origins,
                enabled_redirect_uris: redirect_uris.remove(&client_id).unwrap_or_default(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    struct Fixture {
        repository: PostgresWebOriginRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("web_origin_repository_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
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

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        Fixture {
            repository: PostgresWebOriginRepository::new(db),
            pool,
        }
    }

    async fn insert_realm(pool: &sqlx::PgPool, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("insert realm");
        id
    }

    async fn insert_client(pool: &sqlx::PgPool, realm_id: Uuid, enabled: bool) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, \
             service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 'openid-connect', true, false, 'public', NOW(), NOW())",
        )
        .bind(id)
        .bind(realm_id)
        .bind(format!("client-{id}"))
        .bind(format!("client-{id}"))
        .bind(enabled)
        .execute(pool)
        .await
        .expect("insert client");
        id
    }

    async fn insert_redirect_uri(pool: &sqlx::PgPool, client_id: Uuid, value: &str, enabled: bool) {
        sqlx::query(
            "INSERT INTO redirect_uris (id, client_id, value, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(client_id)
        .bind(value)
        .bind(enabled)
        .execute(pool)
        .await
        .expect("insert redirect uri");
    }

    fn explicit(value: &str) -> WebOriginValue {
        WebOriginValue::Explicit(value.try_into().expect("valid test origin"))
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn origins_of_one_realm_never_leak_into_another() {
        let fixture = setup().await;
        let realm_a = insert_realm(&fixture.pool, "tenant-a").await;
        let realm_b = insert_realm(&fixture.pool, "tenant-b").await;
        let client_a = insert_client(&fixture.pool, realm_a, true).await;
        let client_b = insert_client(&fixture.pool, realm_b, true).await;

        fixture
            .repository
            .create(client_a, explicit("https://a.example.com"))
            .await
            .expect("create origin for tenant a");
        fixture
            .repository
            .create(client_b, explicit("https://b.example.com"))
            .await
            .expect("create origin for tenant b");

        let sources = fixture
            .repository
            .get_origin_sources_by_realm_name("tenant-a".to_string())
            .await
            .expect("resolve tenant a");

        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0].web_origins,
            vec![explicit("https://a.example.com")]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_disabled_client_contributes_no_origin() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-disabled").await;
        let client = insert_client(&fixture.pool, realm, false).await;

        fixture
            .repository
            .create(client, explicit("https://disabled.example.com"))
            .await
            .expect("create origin");

        let sources = fixture
            .repository
            .get_origin_sources_by_realm_name("tenant-disabled".to_string())
            .await
            .expect("resolve realm");

        assert!(sources.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn only_enabled_redirect_uris_are_supplied_for_the_sentinel() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-sentinel").await;
        let client = insert_client(&fixture.pool, realm, true).await;

        insert_redirect_uri(&fixture.pool, client, "https://live.example.com/cb", true).await;
        insert_redirect_uri(
            &fixture.pool,
            client,
            "https://retired.example.com/cb",
            false,
        )
        .await;

        fixture
            .repository
            .create(client, WebOriginValue::DerivedFromRedirectUris)
            .await
            .expect("create sentinel");

        let sources = fixture
            .repository
            .get_origin_sources_by_realm_name("tenant-sentinel".to_string())
            .await
            .expect("resolve realm");

        assert_eq!(
            sources[0].enabled_redirect_uris,
            vec!["https://live.example.com/cb".to_string()]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_unknown_realm_resolves_to_nothing() {
        let fixture = setup().await;

        let sources = fixture
            .repository
            .get_origin_sources_by_realm_name("no-such-realm".to_string())
            .await
            .expect("resolve unknown realm");

        assert!(sources.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn registering_the_same_origin_twice_is_a_client_error() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-duplicate").await;
        let client = insert_client(&fixture.pool, realm, true).await;

        fixture
            .repository
            .create(client, explicit("https://app.example.com"))
            .await
            .expect("first insert succeeds");

        let error = fixture
            .repository
            .create(client, explicit("https://app.example.com"))
            .await
            .expect_err("the unique index must reject the second insert");

        assert!(
            matches!(error, CoreError::InvalidWebOrigin(_)),
            "a duplicate is the administrator's mistake, not a server fault, got {error:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn deleting_is_scoped_to_the_owning_client() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-delete-scope").await;
        let owner = insert_client(&fixture.pool, realm, true).await;
        let other = insert_client(&fixture.pool, realm, true).await;

        let origin = fixture
            .repository
            .create(owner, explicit("https://owned.example.com"))
            .await
            .expect("create origin");

        let error = fixture
            .repository
            .delete(other, origin.id)
            .await
            .expect_err("another client must not delete this origin");

        assert!(matches!(error, CoreError::WebOriginNotFound));
        assert_eq!(
            fixture
                .repository
                .get_by_client_id(owner)
                .await
                .expect("list origins")
                .len(),
            1
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn deleting_a_client_takes_its_origins_with_it() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-cascade").await;
        let client = insert_client(&fixture.pool, realm, true).await;

        fixture
            .repository
            .create(client, explicit("https://cascade.example.com"))
            .await
            .expect("create origin");

        sqlx::query("DELETE FROM clients WHERE id = $1")
            .bind(client)
            .execute(&fixture.pool)
            .await
            .expect("delete client");

        assert!(
            fixture
                .repository
                .get_by_client_id(client)
                .await
                .expect("list origins")
                .is_empty()
        );
    }
}
