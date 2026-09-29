use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::client::entities::token_exchange_policy::{
    TokenExchangePolicy, TokenExchangePolicyDefinition,
};
use crate::domain::client::ports::TokenExchangePolicyRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{RealmId, Unscoped};
use crate::entity::token_exchange_policies::{
    ActiveModel, Column as PolicyColumn, Entity as PolicyEntity, Model as PolicyModel,
};
use crate::infrastructure::common::is_unique_violation;

fn scopes_to_column(scopes: Option<&[String]>) -> Option<String> {
    scopes.map(|scopes| scopes.join(" "))
}

fn scopes_from_column(column: Option<String>) -> Option<Vec<String>> {
    column.map(|value| value.split_whitespace().map(str::to_owned).collect())
}

impl From<PolicyModel> for TokenExchangePolicy {
    fn from(model: PolicyModel) -> Self {
        Self {
            id: model.id,
            realm_id: RealmId::new(model.realm_id),
            client_id: model.client_id,
            target_audience: model.target_audience,
            allowed_scopes: scopes_from_column(model.allowed_scopes),
            allow_impersonation: model.allow_impersonation,
            allow_delegation: model.allow_delegation,
            created_at: DateTime::<Utc>::from_naive_utc_and_offset(model.created_at, Utc),
            updated_at: DateTime::<Utc>::from_naive_utc_and_offset(model.updated_at, Utc),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresTokenExchangePolicyRepository {
    pub db: DatabaseConnection,
}

impl PostgresTokenExchangePolicyRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl TokenExchangePolicyRepository for PostgresTokenExchangePolicyRepository {
    async fn create(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        definition: TokenExchangePolicyDefinition,
    ) -> Result<TokenExchangePolicy, CoreError> {
        let policy = TokenExchangePolicy::new(realm_id, client_id, definition);

        let payload = ActiveModel {
            id: Set(policy.id),
            realm_id: Set(policy.realm_id.into()),
            client_id: Set(policy.client_id),
            target_audience: Set(policy.target_audience.clone()),
            allowed_scopes: Set(scopes_to_column(policy.allowed_scopes.as_deref())),
            allow_impersonation: Set(policy.allow_impersonation),
            allow_delegation: Set(policy.allow_delegation),
            created_at: Set(policy.created_at.naive_utc()),
            updated_at: Set(policy.updated_at.naive_utc()),
        };

        let inserted = payload.insert(&self.db).await.map_err(|e| {
            if is_unique_violation(&e) {
                return CoreError::AlreadyExists;
            }
            error!("Error creating token exchange policy: {e:?}");
            CoreError::InternalServerError
        })?;

        Ok(inserted.into())
    }

    async fn list_by_client(&self, client_id: Uuid) -> Result<Vec<TokenExchangePolicy>, CoreError> {
        let rows = PolicyEntity::find()
            .filter(PolicyColumn::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Error listing token exchange policies: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_for_target(
        &self,
        client_id: Uuid,
        target_audience: String,
    ) -> Result<Option<Unscoped<TokenExchangePolicy>>, CoreError> {
        let row = PolicyEntity::find()
            .filter(PolicyColumn::ClientId.eq(client_id))
            .filter(PolicyColumn::TargetAudience.eq(target_audience))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error looking up token exchange policy: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(row.map(|model| Unscoped::new(model.into())))
    }

    async fn delete(&self, client_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = PolicyEntity::delete_many()
            .filter(PolicyColumn::Id.eq(id))
            .filter(PolicyColumn::ClientId.eq(client_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting token exchange policy: {e:?}");
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    use crate::domain::realm::entities::{Realm, RealmScope};

    #[test]
    fn absent_scopes_stay_absent() {
        assert_eq!(scopes_to_column(None), None);
        assert_eq!(scopes_from_column(None), None);
    }

    #[test]
    fn scopes_round_trip_through_the_column() {
        for scopes in [
            vec![],
            vec!["profile".to_string()],
            vec![
                "profile".to_string(),
                "email".to_string(),
                "orders:read".to_string(),
            ],
        ] {
            let column = scopes_to_column(Some(&scopes));
            assert_eq!(scopes_from_column(column), Some(scopes));
        }
    }

    struct Fixture {
        repository: PostgresTokenExchangePolicyRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("token_exchange_policy_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresTokenExchangePolicyRepository::new(db),
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

    async fn insert_client(pool: &sqlx::PgPool, realm_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, \
             service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, true, 'openid-connect', false, false, 'confidential', NOW(), NOW())",
        )
        .bind(id)
        .bind(realm_id)
        .bind(format!("client-{id}"))
        .bind(format!("client-{id}"))
        .execute(pool)
        .await
        .expect("insert client");
        id
    }

    fn definition(target_audience: &str) -> TokenExchangePolicyDefinition {
        TokenExchangePolicyDefinition {
            target_audience: target_audience.to_string(),
            allowed_scopes: Some(vec!["orders:read".to_string()]),
            allow_impersonation: true,
            allow_delegation: false,
        }
    }

    fn scope_of(realm_id: Uuid, name: &str) -> RealmScope {
        let mut realm = Realm::new(name.to_string());
        realm.id = RealmId::new(realm_id);
        RealmScope::from_realm(realm)
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_created_policy_is_found_for_its_target() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-a").await;
        let client = insert_client(&fixture.pool, realm).await;

        let created = fixture
            .repository
            .create(RealmId::new(realm), client, definition("orders-api"))
            .await
            .expect("create policy");

        let found = fixture
            .repository
            .find_for_target(client, "orders-api".to_string())
            .await
            .expect("look up policy")
            .expect("policy exists")
            .in_realm(&scope_of(realm, "tenant-a"))
            .expect("policy belongs to the realm")
            .into_inner();

        assert_eq!(found.id, created.id);
        assert_eq!(found.allowed_scopes, Some(vec!["orders:read".to_string()]));
        assert!(found.allow_impersonation);
        assert!(!found.allow_delegation);

        let missing = fixture
            .repository
            .find_for_target(client, "billing-api".to_string())
            .await
            .expect("look up policy");
        assert!(missing.is_none());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_policy_is_refused_in_another_realm() {
        let fixture = setup().await;
        let realm_a = insert_realm(&fixture.pool, "tenant-a").await;
        let realm_b = insert_realm(&fixture.pool, "tenant-b").await;
        let client = insert_client(&fixture.pool, realm_a).await;

        fixture
            .repository
            .create(RealmId::new(realm_a), client, definition("orders-api"))
            .await
            .expect("create policy");

        let found = fixture
            .repository
            .find_for_target(client, "orders-api".to_string())
            .await
            .expect("look up policy")
            .expect("policy exists")
            .in_realm(&scope_of(realm_b, "tenant-b"));

        assert!(matches!(found, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_duplicate_target_is_a_conflict() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-a").await;
        let client = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .create(RealmId::new(realm), client, definition("orders-api"))
            .await
            .expect("create policy");

        let duplicate = fixture
            .repository
            .create(RealmId::new(realm), client, definition("orders-api"))
            .await;

        assert!(matches!(duplicate, Err(CoreError::AlreadyExists)));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn policies_are_listed_and_deleted_per_client() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-a").await;
        let client = insert_client(&fixture.pool, realm).await;
        let other = insert_client(&fixture.pool, realm).await;

        let orders = fixture
            .repository
            .create(RealmId::new(realm), client, definition("orders-api"))
            .await
            .expect("create policy");
        fixture
            .repository
            .create(RealmId::new(realm), client, definition("billing-api"))
            .await
            .expect("create policy");
        fixture
            .repository
            .create(RealmId::new(realm), other, definition("orders-api"))
            .await
            .expect("create policy");

        let listed = fixture
            .repository
            .list_by_client(client)
            .await
            .expect("list policies");
        assert_eq!(listed.len(), 2);

        let foreign_delete = fixture.repository.delete(other, orders.id).await;
        assert!(matches!(foreign_delete, Err(CoreError::NotFound)));

        fixture
            .repository
            .delete(client, orders.id)
            .await
            .expect("delete policy");

        let remaining = fixture
            .repository
            .list_by_client(client)
            .await
            .expect("list policies");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].target_audience, "billing-api");
    }
}
