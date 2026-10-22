use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use ferriskey_security::crypto::secret_cipher::SecretCipher;
use ferriskey_security::jwt::ports::KeyStoreRepository;
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::{Expr, OnConflict},
};
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;
use crate::domain::{
    common::generate_uuid_v7,
    jwt::{JwtError, entities::JwtKeyPair},
};
use crate::entity::{jwt_keys, realms};

pub const PRIVATE_KEY_PURPOSE: &str = "jwt_keys.private_key";

#[derive(Clone)]
pub struct PostgresKeyStoreRepository {
    pub db: DatabaseConnection,
    keys: Arc<RwLock<HashMap<Uuid, JwtKeyPair>>>,
    cipher: SecretCipher,
}

impl fmt::Debug for PostgresKeyStoreRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresKeyStoreRepository")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl PostgresKeyStoreRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_cipher(db, SecretCipher::disabled())
    }

    pub fn with_cipher(db: DatabaseConnection, cipher: SecretCipher) -> Self {
        Self {
            db,
            keys: Arc::new(RwLock::new(HashMap::new())),
            cipher,
        }
    }

    fn private_key_of(&self, key: &jwt_keys::Model) -> Result<String, JwtError> {
        self.cipher
            .open(PRIVATE_KEY_PURPOSE, &key.id.to_string(), &key.private_key)
            .map_err(|error| JwtError::InvalidKey(error.to_string()))
    }

    fn cached_key(&self, realm_id: Uuid) -> Option<JwtKeyPair> {
        self.keys
            .read()
            .ok()
            .and_then(|keys| keys.get(&realm_id).cloned())
    }

    fn remember_key(&self, realm_id: Uuid, key: &JwtKeyPair) {
        if let Ok(mut keys) = self.keys.write() {
            keys.insert(realm_id, key.clone());
        }
    }

    async fn find_by_realm(&self, realm_id: Uuid) -> Result<Option<jwt_keys::Model>, JwtError> {
        jwt_keys::Entity::find()
            .filter(jwt_keys::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)
    }

    async fn certificate_common_name(&self, realm_id: Uuid) -> Result<String, JwtError> {
        realms::Entity::find_by_id(realm_id)
            .one(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?
            .map(|realm| realm.name)
            .ok_or(JwtError::RealmKeyNotFound)
    }

    async fn generate_key(&self, realm_id: Uuid) -> Result<jwt_keys::Model, JwtError> {
        let common_name = self.certificate_common_name(realm_id).await?;
        let (private_key, public_key) = JwtKeyPair::generate()?;
        let certificate = JwtKeyPair::self_signed_certificate(&private_key, &common_name)?;

        let id = generate_uuid_v7();
        let sealed_private_key = self
            .cipher
            .seal(PRIVATE_KEY_PURPOSE, &id.to_string(), &private_key)
            .map_err(|error| JwtError::GenerationError(error.to_string()))?;

        let new_key = jwt_keys::ActiveModel {
            id: Set(id),
            realm_id: Set(realm_id),
            public_key: Set(public_key),
            private_key: Set(sealed_private_key),
            certificate: Set(Some(certificate)),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        jwt_keys::Entity::insert(new_key)
            .on_conflict(
                OnConflict::column(jwt_keys::Column::RealmId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        self.find_by_realm(realm_id)
            .await?
            .ok_or(JwtError::RealmKeyNotFound)
    }

    async fn backfill_certificate(
        &self,
        key: &jwt_keys::Model,
    ) -> Result<jwt_keys::Model, JwtError> {
        let common_name = self.certificate_common_name(key.realm_id).await?;
        let private_key = self.private_key_of(key)?;
        let certificate = JwtKeyPair::self_signed_certificate(&private_key, &common_name)?;

        jwt_keys::Entity::update_many()
            .col_expr(jwt_keys::Column::Certificate, Expr::value(certificate))
            .filter(jwt_keys::Column::Id.eq(key.id))
            .filter(jwt_keys::Column::Certificate.is_null())
            .exec(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        self.find_by_realm(key.realm_id)
            .await?
            .ok_or(JwtError::RealmKeyNotFound)
    }
}

impl KeyStoreRepository for PostgresKeyStoreRepository {
    async fn get_or_generate_key(&self, realm_id: RealmId) -> Result<JwtKeyPair, JwtError> {
        let realm_id: Uuid = realm_id.into();

        if let Some(key) = self.cached_key(realm_id) {
            return Ok(key);
        }

        let key = match self.find_by_realm(realm_id).await? {
            Some(key) => key,
            None => self.generate_key(realm_id).await?,
        };

        let key = if key.certificate.is_some() {
            key
        } else {
            self.backfill_certificate(&key).await?
        };

        let private_key = self.private_key_of(&key)?;
        let certificate = key.certificate.ok_or(JwtError::RealmKeyNotFound)?;

        let key_pair = JwtKeyPair::from_pem(
            &private_key,
            &key.public_key,
            &certificate,
            key.realm_id,
            key.id,
        )?;
        self.remember_key(realm_id, &key_pair);

        Ok(key_pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use x509_parser::pem::parse_x509_pem;

    struct Fixture {
        repository: PostgresKeyStoreRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("keystore_repository_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresKeyStoreRepository::new(db),
            pool,
        }
    }

    async fn insert_realm(pool: &sqlx::PgPool, name: &str) -> RealmId {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("insert realm");
        RealmId::new(id)
    }

    async fn count_keys(pool: &sqlx::PgPool, realm_id: RealmId) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM jwt_keys WHERE realm_id = $1")
            .bind(Uuid::from(realm_id))
            .fetch_one(pool)
            .await
            .expect("count keys")
    }

    fn common_name_of(certificate_pem: &str) -> String {
        let (_, pem) =
            parse_x509_pem(certificate_pem.as_bytes()).expect("certificate is valid PEM");
        pem.parse_x509()
            .expect("certificate is valid X.509")
            .subject()
            .to_string()
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_realm_key_is_generated_once_with_a_certificate_for_the_realm() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-alpha").await;

        let first = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("generate realm key");

        assert_eq!(common_name_of(&first.certificate), "CN=tenant-alpha");
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_realm_key_is_stable_across_calls() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-beta").await;

        let first = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("generate realm key");
        let second = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("read realm key");

        assert_eq!(first.id, second.id);
        assert_eq!(first.private_key, second.private_key);
        assert_eq!(first.certificate, second.certificate);
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_key_predating_certificates_is_backfilled_in_place() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-legacy").await;

        let (private_key, public_key) = JwtKeyPair::generate().expect("generate legacy key");
        let legacy_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO jwt_keys (id, realm_id, private_key, public_key, created_at) \
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(legacy_id)
        .bind(Uuid::from(realm_id))
        .bind(&private_key)
        .bind(&public_key)
        .execute(&fixture.pool)
        .await
        .expect("insert legacy key");

        let key_pair = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("backfill certificate");

        assert_eq!(key_pair.id, legacy_id);
        assert_eq!(key_pair.private_key, private_key);
        assert_eq!(common_name_of(&key_pair.certificate), "CN=tenant-legacy");
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 1);

        let persisted: Option<String> =
            sqlx::query_scalar("SELECT certificate FROM jwt_keys WHERE id = $1")
                .bind(legacy_id)
                .fetch_one(&fixture.pool)
                .await
                .expect("read back certificate");
        assert_eq!(persisted.as_deref(), Some(key_pair.certificate.as_str()));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn keys_of_two_realms_stay_distinct() {
        let fixture = setup().await;
        let alpha = insert_realm(&fixture.pool, "tenant-one").await;
        let beta = insert_realm(&fixture.pool, "tenant-two").await;

        let alpha_key = fixture
            .repository
            .get_or_generate_key(alpha)
            .await
            .expect("generate key for tenant-one");
        let beta_key = fixture
            .repository
            .get_or_generate_key(beta)
            .await
            .expect("generate key for tenant-two");

        assert_ne!(alpha_key.certificate, beta_key.certificate);
        assert_eq!(common_name_of(&alpha_key.certificate), "CN=tenant-one");
        assert_eq!(common_name_of(&beta_key.certificate), "CN=tenant-two");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_loaded_key_is_served_from_memory_afterwards() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "cached").await;

        let first = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first load");
        sqlx::query("DELETE FROM jwt_keys WHERE realm_id = $1")
            .bind(Uuid::from(realm_id))
            .execute(&fixture.pool)
            .await
            .expect("delete the stored key");

        let second = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("second load");

        assert_eq!(second.id, first.id);
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 0);
    }

    const SEALING_KEY: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

    fn sealing(db: sea_orm::DatabaseConnection) -> PostgresKeyStoreRepository {
        PostgresKeyStoreRepository::with_cipher(
            db,
            ferriskey_security::crypto::secret_cipher::SecretCipher::from_config(SEALING_KEY)
                .expect("a valid key ring"),
        )
    }

    async fn stored_private_key(pool: &sqlx::PgPool, realm_id: RealmId) -> String {
        sqlx::query_scalar("SELECT private_key FROM jwt_keys WHERE realm_id = $1")
            .bind(Uuid::from(realm_id))
            .fetch_one(pool)
            .await
            .expect("read the stored key")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn with_a_cipher_the_private_key_is_sealed_at_rest_and_still_usable() {
        let fixture = setup().await;
        let repository = sealing(fixture.repository.db.clone());
        let realm_id = insert_realm(&fixture.pool, "tenant-sealed").await;

        let key_pair = repository
            .get_or_generate_key(realm_id)
            .await
            .expect("generate realm key");

        let stored = stored_private_key(&fixture.pool, realm_id).await;
        assert!(stored.starts_with("fk1:kid-a:"), "{stored}");
        assert!(!stored.contains("PRIVATE KEY"));
        assert!(key_pair.private_key.contains("PRIVATE KEY"));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sealed_key_is_read_back_by_another_instance_with_the_same_key() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-shared").await;

        let first = sealing(fixture.repository.db.clone())
            .get_or_generate_key(realm_id)
            .await
            .expect("generate realm key");
        let second = sealing(fixture.repository.db.clone())
            .get_or_generate_key(realm_id)
            .await
            .expect("read the sealed key");

        assert_eq!(first.private_key, second.private_key);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_plaintext_key_is_still_read_when_a_cipher_is_configured() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-plain").await;
        let (private_key, public_key) = JwtKeyPair::generate().expect("generate legacy key");
        sqlx::query(
            "INSERT INTO jwt_keys (id, realm_id, private_key, public_key, certificate, created_at) \
             VALUES ($1, $2, $3, $4, $5, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::from(realm_id))
        .bind(&private_key)
        .bind(&public_key)
        .bind(
            JwtKeyPair::self_signed_certificate(&private_key, "tenant-plain").expect("certificate"),
        )
        .execute(&fixture.pool)
        .await
        .expect("insert legacy key");

        let key_pair = sealing(fixture.repository.db.clone())
            .get_or_generate_key(realm_id)
            .await
            .expect("read the legacy key");

        assert_eq!(key_pair.private_key, private_key);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sealed_key_is_refused_by_an_instance_without_a_cipher() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-locked").await;
        sealing(fixture.repository.db.clone())
            .get_or_generate_key(realm_id)
            .await
            .expect("generate realm key");

        let unaware = PostgresKeyStoreRepository::new(fixture.repository.db.clone());

        assert!(unaware.get_or_generate_key(realm_id).await.is_err());
    }
}
