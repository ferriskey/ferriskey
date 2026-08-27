use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    SqlErr, sea_query::OnConflict,
};
use uuid::Uuid;

use crate::domain::client::{
    entities::saml::{
        ClientSamlConfig, SamlAttributeMapper, SamlAttributeMapperDefinition, SamlConfigSettings,
    },
    ports::ClientSamlRepository,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::entity::{
    client_saml_attribute_mappers::{
        ActiveModel as AttributeMapperActiveModel, Column as AttributeMapperColumn,
        Entity as AttributeMapperEntity,
    },
    client_saml_configs::{
        ActiveModel as SamlConfigActiveModel, Column as SamlConfigColumn,
        Entity as SamlConfigEntity,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresClientSamlRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientSamlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientSamlRepository for PostgresClientSamlRepository {
    async fn get_config_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Option<ClientSamlConfig>, CoreError> {
        SamlConfigEntity::find_by_id(client_id)
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(ClientSamlConfig::try_from)
            .transpose()
    }

    async fn upsert_config(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        settings: SamlConfigSettings,
    ) -> Result<ClientSamlConfig, CoreError> {
        let config = ClientSamlConfig::new(realm_id, client_id, settings);

        let payload = SamlConfigActiveModel {
            client_id: Set(config.client_id),
            realm_id: Set(config.realm_id.into()),
            sp_entity_id: Set(config.sp_entity_id.to_string()),
            acs_url: Set(config.acs_url.to_string()),
            name_id_format: Set(config.name_id_format.to_string()),
            sign_assertions: Set(config.sign_assertions),
            sign_documents: Set(config.sign_documents),
            want_authn_requests_signed: Set(config.want_authn_requests_signed),
            created_at: Set(config.created_at.naive_utc()),
            updated_at: Set(config.updated_at.naive_utc()),
        };

        let persisted = SamlConfigEntity::insert(payload)
            .on_conflict(
                OnConflict::column(SamlConfigColumn::ClientId)
                    .update_columns([
                        SamlConfigColumn::SpEntityId,
                        SamlConfigColumn::AcsUrl,
                        SamlConfigColumn::NameIdFormat,
                        SamlConfigColumn::SignAssertions,
                        SamlConfigColumn::SignDocuments,
                        SamlConfigColumn::WantAuthnRequestsSigned,
                        SamlConfigColumn::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|error| match error.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => CoreError::InvalidSamlConfig(
                    "this service provider entity id is already registered in the realm"
                        .to_string(),
                ),
                _ => CoreError::InternalServerError,
            })?;

        ClientSamlConfig::try_from(persisted)
    }

    async fn create_attribute_mapper(
        &self,
        client_id: Uuid,
        definition: SamlAttributeMapperDefinition,
    ) -> Result<SamlAttributeMapper, CoreError> {
        let mapper = SamlAttributeMapper::new(client_id, definition);

        let payload = AttributeMapperActiveModel {
            id: Set(mapper.id),
            client_id: Set(mapper.client_id),
            name: Set(mapper.name.to_string()),
            name_format: Set(mapper.name_format.to_string()),
            source: Set(mapper.source.to_string()),
            created_at: Set(mapper.created_at.naive_utc()),
            updated_at: Set(mapper.updated_at.naive_utc()),
        };

        let inserted = AttributeMapperEntity::insert(payload)
            .exec_with_returning(&self.db)
            .await
            .map_err(|error| match error.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => {
                    CoreError::InvalidSamlAttributeMapper(
                        "this attribute name is already mapped for the client".to_string(),
                    )
                }
                _ => CoreError::InternalServerError,
            })?;

        SamlAttributeMapper::try_from(inserted)
    }

    async fn get_attribute_mappers_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<SamlAttributeMapper>, CoreError> {
        AttributeMapperEntity::find()
            .filter(AttributeMapperColumn::ClientId.eq(client_id))
            .order_by_asc(AttributeMapperColumn::Name)
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into_iter()
            .map(SamlAttributeMapper::try_from)
            .collect()
    }

    async fn delete_attribute_mapper(
        &self,
        client_id: Uuid,
        mapper_id: Uuid,
    ) -> Result<(), CoreError> {
        let result = AttributeMapperEntity::delete_many()
            .filter(AttributeMapperColumn::Id.eq(mapper_id))
            .filter(AttributeMapperColumn::ClientId.eq(client_id))
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if result.rows_affected == 0 {
            return Err(CoreError::SamlAttributeMapperNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::client::entities::saml::{
        AcsUrl, NameIdFormat, SamlAttributeName, SamlAttributeNameFormat, SamlAttributeSource,
        SpEntityId,
    };
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use std::str::FromStr;

    struct Fixture {
        repository: PostgresClientSamlRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("client_saml_repository_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresClientSamlRepository::new(db),
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
             VALUES ($1, $2, $3, $4, true, 'saml', false, false, 'confidential', NOW(), NOW())",
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

    fn chatwoot_settings(account_id: u32) -> SamlConfigSettings {
        SamlConfigSettings {
            sp_entity_id: SpEntityId::from_str(&format!(
                "https://chat.example.com/saml/sp/{account_id}"
            ))
            .expect("a valid entity id"),
            acs_url: AcsUrl::from_str(&format!(
                "https://chat.example.com/omniauth/saml/callback?account_id={account_id}"
            ))
            .expect("a valid acs url"),
            name_id_format: NameIdFormat::EmailAddress,
            sign_assertions: true,
            sign_documents: false,
            want_authn_requests_signed: false,
        }
    }

    fn mapper(name: &str, source: SamlAttributeSource) -> SamlAttributeMapperDefinition {
        SamlAttributeMapperDefinition {
            name: SamlAttributeName::from_str(name).expect("a valid attribute name"),
            name_format: SamlAttributeNameFormat::Basic,
            source,
        }
    }

    fn settings_claiming(entity_id: &str) -> SamlConfigSettings {
        SamlConfigSettings {
            sp_entity_id: SpEntityId::from_str(entity_id).expect("a valid entity id"),
            ..chatwoot_settings(1)
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn two_clients_of_one_realm_cannot_claim_the_same_entity_id() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-entity-id-clash").await;
        let first = insert_client(&fixture.pool, realm).await;
        let second = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .upsert_config(
                realm.into(),
                first,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect("the first client claims the entity id");

        let error = fixture
            .repository
            .upsert_config(
                realm.into(),
                second,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect_err("a second client must not claim the same entity id in the realm");

        assert!(
            matches!(error, CoreError::InvalidSamlConfig(_)),
            "an ambiguous issuer would route an assertion to the wrong service provider, \
             and it is the administrator's mistake, not a server fault, got {error:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn two_clients_of_different_realms_may_claim_the_same_entity_id() {
        let fixture = setup().await;
        let realm_a = insert_realm(&fixture.pool, "tenant-entity-id-a").await;
        let realm_b = insert_realm(&fixture.pool, "tenant-entity-id-b").await;
        let client_a = insert_client(&fixture.pool, realm_a).await;
        let client_b = insert_client(&fixture.pool, realm_b).await;

        fixture
            .repository
            .upsert_config(
                realm_a.into(),
                client_a,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect("tenant a claims the entity id");

        fixture
            .repository
            .upsert_config(
                realm_b.into(),
                client_b,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect("uniqueness is scoped to the realm, not global");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_client_may_keep_its_own_entity_id_when_its_config_is_rewritten() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-entity-id-rewrite").await;
        let client = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .upsert_config(
                realm.into(),
                client,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect("the client claims the entity id");

        fixture
            .repository
            .upsert_config(
                realm.into(),
                client,
                settings_claiming("https://sp.example.com"),
            )
            .await
            .expect("the realm-scoped index must not fire against the row it replaces");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn mapping_the_same_attribute_name_twice_is_a_client_error() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-duplicate-mapper").await;
        let client = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .create_attribute_mapper(client, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("first insert succeeds");

        let error = fixture
            .repository
            .create_attribute_mapper(client, mapper("email", SamlAttributeSource::Username))
            .await
            .expect_err("the unique index must reject the second insert");

        assert!(
            matches!(error, CoreError::InvalidSamlAttributeMapper(_)),
            "a duplicate is the administrator's mistake, not a server fault, got {error:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_same_attribute_name_may_be_mapped_by_two_different_clients() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-shared-name").await;
        let first = insert_client(&fixture.pool, realm).await;
        let second = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .create_attribute_mapper(first, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("first client maps email");

        fixture
            .repository
            .create_attribute_mapper(second, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("the unique index is scoped to the client");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_client_carries_no_config_until_one_is_written() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-unconfigured").await;
        let client = insert_client(&fixture.pool, realm).await;

        assert!(
            fixture
                .repository
                .get_config_by_client_id(client)
                .await
                .expect("read config")
                .is_none()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn writing_a_config_twice_replaces_it_rather_than_failing() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-upsert").await;
        let client = insert_client(&fixture.pool, realm).await;

        let first = fixture
            .repository
            .upsert_config(realm.into(), client, chatwoot_settings(1))
            .await
            .expect("write the first config");

        let second = fixture
            .repository
            .upsert_config(realm.into(), client, chatwoot_settings(2))
            .await
            .expect("overwrite the config");

        assert_eq!(
            second.acs_url.as_str(),
            "https://chat.example.com/omniauth/saml/callback?account_id=2"
        );
        assert_eq!(
            second.created_at, first.created_at,
            "an overwrite keeps the moment the client was first configured"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_config_round_trips_through_postgres() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-round-trip").await;
        let client = insert_client(&fixture.pool, realm).await;

        let written = fixture
            .repository
            .upsert_config(realm.into(), client, chatwoot_settings(7))
            .await
            .expect("write the config");

        let read = fixture
            .repository
            .get_config_by_client_id(client)
            .await
            .expect("read the config")
            .expect("the config exists");

        assert_eq!(read, written);
        assert_eq!(read.name_id_format, NameIdFormat::EmailAddress);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_custom_attribute_source_round_trips_through_postgres() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-custom-source").await;
        let client = insert_client(&fixture.pool, realm).await;

        let source =
            SamlAttributeSource::from_str("attribute:department").expect("a valid custom source");

        fixture
            .repository
            .create_attribute_mapper(client, mapper("department", source.clone()))
            .await
            .expect("create the mapper");

        let mappers = fixture
            .repository
            .get_attribute_mappers_by_client_id(client)
            .await
            .expect("list mappers");

        assert_eq!(mappers.len(), 1);
        assert_eq!(mappers[0].source, source);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn mappers_of_one_client_never_leak_into_another() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-mapper-isolation").await;
        let owner = insert_client(&fixture.pool, realm).await;
        let other = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .create_attribute_mapper(owner, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("create the mapper");

        assert!(
            fixture
                .repository
                .get_attribute_mappers_by_client_id(other)
                .await
                .expect("list mappers")
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn deleting_a_mapper_is_scoped_to_the_owning_client() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-mapper-delete-scope").await;
        let owner = insert_client(&fixture.pool, realm).await;
        let other = insert_client(&fixture.pool, realm).await;

        let created = fixture
            .repository
            .create_attribute_mapper(owner, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("create the mapper");

        let error = fixture
            .repository
            .delete_attribute_mapper(other, created.id)
            .await
            .expect_err("another client must not delete this mapper");

        assert!(matches!(error, CoreError::SamlAttributeMapperNotFound));
        assert_eq!(
            fixture
                .repository
                .get_attribute_mappers_by_client_id(owner)
                .await
                .expect("list mappers")
                .len(),
            1
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn deleting_a_client_takes_its_saml_setup_with_it() {
        let fixture = setup().await;
        let realm = insert_realm(&fixture.pool, "tenant-saml-cascade").await;
        let client = insert_client(&fixture.pool, realm).await;

        fixture
            .repository
            .upsert_config(realm.into(), client, chatwoot_settings(1))
            .await
            .expect("write the config");
        fixture
            .repository
            .create_attribute_mapper(client, mapper("email", SamlAttributeSource::Email))
            .await
            .expect("create the mapper");

        sqlx::query("DELETE FROM clients WHERE id = $1")
            .bind(client)
            .execute(&fixture.pool)
            .await
            .expect("delete client");

        assert!(
            fixture
                .repository
                .get_config_by_client_id(client)
                .await
                .expect("read config")
                .is_none()
        );
        assert!(
            fixture
                .repository
                .get_attribute_mappers_by_client_id(client)
                .await
                .expect("list mappers")
                .is_empty()
        );
    }
}
