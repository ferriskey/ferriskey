use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use ferriskey_security::crypto::secret_cipher::SecretCipher;
use ferriskey_security::jwt::ports::{KeyStoreRepository, SigningKeyRepository};
use ferriskey_security::jwt::signing_keys::{
    PassiveKey, RealmKeySet, SigningKey, SigningKeyLifecycle,
};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
    sea_query::{Expr, OnConflict},
};
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;
use crate::domain::{
    common::generate_uuid_v7,
    jwt::{JwtError, entities::JwtKeyPair},
};
use crate::entity::{jwt_keys, realms};
use crate::infrastructure::token_state::TokenState;

pub const PRIVATE_KEY_PURPOSE: &str = "jwt_keys.private_key";

/// How long a replica trusts its copy of a realm's key set when it may have
/// missed a change announced by another replica.
pub const KEY_SET_TTL: Duration = Duration::from_secs(30);

struct CachedKeySet {
    set: Arc<RealmKeySet>,
    generation: u64,
    fresh_until: Option<Instant>,
}

impl CachedKeySet {
    fn is_fresh(&self, generation: u64) -> bool {
        self.generation == generation
            && self
                .fresh_until
                .is_some_and(|fresh_until| Instant::now() < fresh_until)
    }

    fn key(&self, kid: Uuid) -> Option<&JwtKeyPair> {
        self.set.all().find(|key| key.id == kid)
    }
}

#[derive(Clone)]
pub struct PostgresKeyStoreRepository {
    pub db: DatabaseConnection,
    sets: Arc<RwLock<HashMap<Uuid, CachedKeySet>>>,
    cipher: SecretCipher,
    changes: Option<Arc<TokenState>>,
}

impl fmt::Debug for PostgresKeyStoreRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresKeyStoreRepository")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

fn utc(timestamp: chrono::NaiveDateTime) -> DateTime<Utc> {
    timestamp.and_utc()
}

fn lifecycle_of(key: &jwt_keys::Model) -> SigningKeyLifecycle {
    SigningKeyLifecycle {
        rotated_at: key.rotated_at.map(utc),
        retire_at: key.retire_at.map(utc),
        revoked_at: key.revoked_at.map(utc),
    }
}

fn signing_key_of(key: &jwt_keys::Model, now: DateTime<Utc>) -> SigningKey {
    SigningKey::new(
        key.id,
        key.realm_id,
        utc(key.created_at),
        lifecycle_of(key),
        now,
    )
}

fn storage_error(error: sea_orm::DbErr) -> JwtError {
    JwtError::GenerationError(error.to_string())
}

fn active_key_filter() -> sea_orm::Condition {
    sea_orm::Condition::all()
        .add(jwt_keys::Column::RotatedAt.is_null())
        .add(jwt_keys::Column::RevokedAt.is_null())
}

impl PostgresKeyStoreRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_cipher(db, SecretCipher::disabled())
    }

    pub fn with_cipher(db: DatabaseConnection, cipher: SecretCipher) -> Self {
        Self {
            db,
            sets: Arc::new(RwLock::new(HashMap::new())),
            cipher,
            changes: None,
        }
    }

    /// Lets the other replicas drop their copy of a key set as soon as this one
    /// rotates or revokes a key, instead of waiting for [`KEY_SET_TTL`].
    pub fn announcing_changes(mut self, changes: Arc<TokenState>) -> Self {
        self.changes = Some(changes);
        self
    }

    fn generation(&self) -> u64 {
        self.changes
            .as_ref()
            .map_or(0, |changes| changes.generation())
    }

    fn private_key_of(&self, key: &jwt_keys::Model) -> Result<String, JwtError> {
        self.cipher
            .open(PRIVATE_KEY_PURPOSE, &key.id.to_string(), &key.private_key)
            .map_err(|error| JwtError::InvalidKey(error.to_string()))
    }

    fn fresh_set(&self, realm_id: Uuid) -> Option<Arc<RealmKeySet>> {
        let generation = self.generation();
        self.sets.read().ok().and_then(|sets| {
            sets.get(&realm_id)
                .filter(|cached| cached.is_fresh(generation))
                .map(|cached| Arc::clone(&cached.set))
        })
    }

    fn fresh_active_key(&self, realm_id: Uuid) -> Option<JwtKeyPair> {
        let generation = self.generation();
        self.sets.read().ok().and_then(|sets| {
            sets.get(&realm_id)
                .filter(|cached| cached.is_fresh(generation))
                .map(|cached| cached.set.active().clone())
        })
    }

    fn known_key(&self, realm_id: Uuid, kid: Uuid) -> Option<JwtKeyPair> {
        self.sets.read().ok().and_then(|sets| {
            sets.get(&realm_id)
                .and_then(|cached| cached.key(kid).cloned())
        })
    }

    fn remember_set(&self, realm_id: Uuid, set: &Arc<RealmKeySet>, generation: u64) {
        if let Ok(mut sets) = self.sets.write() {
            sets.insert(
                realm_id,
                CachedKeySet {
                    set: Arc::clone(set),
                    generation,
                    fresh_until: Some(Instant::now() + KEY_SET_TTL),
                },
            );
        }
    }

    fn key_set_changed(&self, realm_id: Uuid) {
        if let Ok(mut sets) = self.sets.write()
            && let Some(cached) = sets.get_mut(&realm_id)
        {
            cached.fresh_until = None;
        }

        if let Some(changes) = &self.changes {
            changes.changed();
        }
    }

    async fn published_rows(
        &self,
        realm_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Vec<jwt_keys::Model>, JwtError> {
        jwt_keys::Entity::find()
            .filter(jwt_keys::Column::RealmId.eq(realm_id))
            .filter(jwt_keys::Column::RevokedAt.is_null())
            .filter(
                sea_orm::Condition::any()
                    .add(jwt_keys::Column::RetireAt.is_null())
                    .add(jwt_keys::Column::RetireAt.gt(now.naive_utc())),
            )
            .order_by_desc(jwt_keys::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)
    }

    async fn certificate_common_name<C: ConnectionTrait>(
        &self,
        connection: &C,
        realm_id: Uuid,
    ) -> Result<String, JwtError> {
        realms::Entity::find_by_id(realm_id)
            .one(connection)
            .await
            .map_err(storage_error)?
            .map(|realm| realm.name)
            .ok_or(JwtError::RealmKeyNotFound)
    }

    fn new_key(
        &self,
        realm_id: Uuid,
        common_name: &str,
        now: DateTime<Utc>,
    ) -> Result<jwt_keys::ActiveModel, JwtError> {
        let (private_key, public_key) = JwtKeyPair::generate()?;
        let certificate = JwtKeyPair::self_signed_certificate(&private_key, common_name)?;

        let id = generate_uuid_v7();
        let sealed_private_key = self
            .cipher
            .seal(PRIVATE_KEY_PURPOSE, &id.to_string(), &private_key)
            .map_err(|error| JwtError::GenerationError(error.to_string()))?;

        Ok(jwt_keys::ActiveModel {
            id: Set(id),
            realm_id: Set(realm_id),
            public_key: Set(public_key),
            private_key: Set(sealed_private_key),
            certificate: Set(Some(certificate)),
            created_at: Set(now.naive_utc()),
            rotated_at: Set(None),
            retire_at: Set(None),
            revoked_at: Set(None),
        })
    }

    async fn generate_first_key(&self, realm_id: Uuid) -> Result<(), JwtError> {
        let common_name = self.certificate_common_name(&self.db, realm_id).await?;
        let new_key = self.new_key(realm_id, &common_name, Utc::now())?;

        jwt_keys::Entity::insert(new_key)
            .on_conflict(
                OnConflict::column(jwt_keys::Column::RealmId)
                    .target_cond_where(active_key_filter())
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(&self.db)
            .await
            .map_err(storage_error)?;

        Ok(())
    }

    async fn backfill_certificate(
        &self,
        key: jwt_keys::Model,
    ) -> Result<jwt_keys::Model, JwtError> {
        let common_name = self.certificate_common_name(&self.db, key.realm_id).await?;
        let private_key = self.private_key_of(&key)?;
        let certificate = JwtKeyPair::self_signed_certificate(&private_key, &common_name)?;

        jwt_keys::Entity::update_many()
            .col_expr(jwt_keys::Column::Certificate, Expr::value(certificate))
            .filter(jwt_keys::Column::Id.eq(key.id))
            .filter(jwt_keys::Column::Certificate.is_null())
            .exec(&self.db)
            .await
            .map_err(storage_error)?;

        jwt_keys::Entity::find_by_id(key.id)
            .one(&self.db)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)?
            .ok_or(JwtError::RealmKeyNotFound)
    }

    async fn key_pair_of(&self, key: jwt_keys::Model) -> Result<JwtKeyPair, JwtError> {
        if let Some(known) = self.known_key(key.realm_id, key.id) {
            return Ok(known);
        }

        let key = if key.certificate.is_some() {
            key
        } else {
            self.backfill_certificate(key).await?
        };

        let private_key = self.private_key_of(&key)?;
        let certificate = key.certificate.ok_or(JwtError::RealmKeyNotFound)?;

        JwtKeyPair::from_pem(
            &private_key,
            &key.public_key,
            &certificate,
            key.realm_id,
            key.id,
        )
    }

    async fn load_set(&self, realm_id: Uuid) -> Result<Arc<RealmKeySet>, JwtError> {
        let generation = self.generation();
        let now = Utc::now();

        let mut rows = self.published_rows(realm_id, now).await?;
        if !rows.iter().any(|row| row.rotated_at.is_none()) {
            self.generate_first_key(realm_id).await?;
            rows = self.published_rows(realm_id, now).await?;
        }

        let mut active = None;
        let mut passive = Vec::with_capacity(rows.len().saturating_sub(1));
        for row in rows {
            let retire_at = row.retire_at.map(utc);
            let key = self.key_pair_of(row).await?;
            match retire_at {
                None => active = Some(key),
                Some(retire_at) => passive.push(PassiveKey { key, retire_at }),
            }
        }

        let set = Arc::new(RealmKeySet::new(
            active.ok_or(JwtError::RealmKeyNotFound)?,
            passive,
        ));
        self.remember_set(realm_id, &set, generation);

        Ok(set)
    }

    async fn lock_realm<C: ConnectionTrait>(
        &self,
        connection: &C,
        realm_id: Uuid,
    ) -> Result<realms::Model, JwtError> {
        realms::Entity::find_by_id(realm_id)
            .lock_exclusive()
            .one(connection)
            .await
            .map_err(storage_error)?
            .ok_or(JwtError::RealmKeyNotFound)
    }
}

impl KeyStoreRepository for PostgresKeyStoreRepository {
    async fn get_or_generate_key(&self, realm_id: RealmId) -> Result<JwtKeyPair, JwtError> {
        let realm_id: Uuid = realm_id.into();

        if let Some(key) = self.fresh_active_key(realm_id) {
            return Ok(key);
        }

        Ok(self.load_set(realm_id).await?.active().clone())
    }

    async fn get_key_set(&self, realm_id: RealmId) -> Result<Arc<RealmKeySet>, JwtError> {
        let realm_id: Uuid = realm_id.into();

        if let Some(set) = self.fresh_set(realm_id) {
            return Ok(set);
        }

        self.load_set(realm_id).await
    }
}

impl SigningKeyRepository for PostgresKeyStoreRepository {
    async fn list_keys(&self, realm_id: RealmId) -> Result<Vec<SigningKey>, JwtError> {
        let now = Utc::now();

        let keys = jwt_keys::Entity::find()
            .filter(jwt_keys::Column::RealmId.eq(Uuid::from(realm_id)))
            .order_by_desc(jwt_keys::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(storage_error)?;

        Ok(keys.iter().map(|key| signing_key_of(key, now)).collect())
    }

    async fn rotate_key(
        &self,
        realm_id: RealmId,
        grace: chrono::Duration,
    ) -> Result<SigningKey, JwtError> {
        let realm_id: Uuid = realm_id.into();
        let now = Utc::now();

        let transaction = self.db.begin().await.map_err(storage_error)?;
        let realm = self.lock_realm(&transaction, realm_id).await?;

        jwt_keys::Entity::update_many()
            .col_expr(jwt_keys::Column::RotatedAt, Expr::value(now.naive_utc()))
            .col_expr(
                jwt_keys::Column::RetireAt,
                Expr::value((now + grace).naive_utc()),
            )
            .filter(jwt_keys::Column::RealmId.eq(realm_id))
            .filter(active_key_filter())
            .exec(&transaction)
            .await
            .map_err(storage_error)?;

        let new_key = jwt_keys::Entity::insert(self.new_key(realm_id, &realm.name, now)?)
            .exec_with_returning(&transaction)
            .await
            .map_err(storage_error)?;

        transaction.commit().await.map_err(storage_error)?;
        self.key_set_changed(realm_id);

        Ok(signing_key_of(&new_key, now))
    }

    async fn revoke_key(
        &self,
        realm_id: RealmId,
        kid: Uuid,
    ) -> Result<Option<SigningKey>, JwtError> {
        let realm_id: Uuid = realm_id.into();
        let now = Utc::now();

        let transaction = self.db.begin().await.map_err(storage_error)?;
        let realm = self.lock_realm(&transaction, realm_id).await?;

        let Some(key) = jwt_keys::Entity::find_by_id(kid)
            .filter(jwt_keys::Column::RealmId.eq(realm_id))
            .one(&transaction)
            .await
            .map_err(storage_error)?
        else {
            return Ok(None);
        };

        if key.revoked_at.is_some() {
            return Ok(Some(signing_key_of(&key, now)));
        }

        let revoked = jwt_keys::Entity::update(jwt_keys::ActiveModel {
            id: Set(key.id),
            revoked_at: Set(Some(now.naive_utc())),
            ..Default::default()
        })
        .exec(&transaction)
        .await
        .map_err(storage_error)?;

        if key.rotated_at.is_none() {
            jwt_keys::Entity::insert(self.new_key(realm_id, &realm.name, now)?)
                .exec_without_returning(&transaction)
                .await
                .map_err(storage_error)?;
        }

        transaction.commit().await.map_err(storage_error)?;
        self.key_set_changed(realm_id);

        Ok(Some(signing_key_of(&revoked, now)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferriskey_security::jwt::signing_keys::SigningKeyStatus;
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

    const DAY: chrono::Duration = chrono::Duration::days(1);

    async fn active_count(pool: &sqlx::PgPool, realm_id: RealmId) -> i64 {
        sqlx::query_scalar(
            "SELECT count(*) FROM jwt_keys \
             WHERE realm_id = $1 AND rotated_at IS NULL AND revoked_at IS NULL",
        )
        .bind(Uuid::from(realm_id))
        .fetch_one(pool)
        .await
        .expect("count active keys")
    }

    fn published(set: &RealmKeySet) -> Vec<Uuid> {
        set.published_at(Utc::now()).map(|key| key.id).collect()
    }

    fn status_of(keys: &[SigningKey], kid: Uuid) -> SigningKeyStatus {
        keys.iter()
            .find(|key| key.kid == kid)
            .map(|key| key.status)
            .expect("the key is listed")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_rotation_signs_with_a_new_key_and_keeps_the_previous_one_published() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-rotated").await;
        let previous = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        let rotated = fixture
            .repository
            .rotate_key(realm_id, DAY)
            .await
            .expect("rotate");

        let active = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("active key");
        let set = fixture
            .repository
            .get_key_set(realm_id)
            .await
            .expect("key set");
        let keys = fixture.repository.list_keys(realm_id).await.expect("list");

        assert_eq!(active.id, rotated.kid);
        assert_eq!(rotated.status, SigningKeyStatus::Active);
        assert_eq!(published(&set), vec![rotated.kid, previous.id]);
        assert_eq!(status_of(&keys, previous.id), SigningKeyStatus::Passive);
        assert_eq!(active_count(&fixture.pool, realm_id).await, 1);
        assert!(
            set.verification_key(Some(&previous.id.to_string()), Utc::now())
                .is_some()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_rotated_key_leaves_the_key_set_once_its_grace_has_passed() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-retired").await;
        let previous = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        let rotated = fixture
            .repository
            .rotate_key(realm_id, chrono::Duration::zero())
            .await
            .expect("rotate");

        let set = fixture
            .repository
            .get_key_set(realm_id)
            .await
            .expect("key set");
        let keys = fixture.repository.list_keys(realm_id).await.expect("list");

        assert_eq!(published(&set), vec![rotated.kid]);
        assert_eq!(status_of(&keys, previous.id), SigningKeyStatus::Retired);
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_a_passive_key_withdraws_it_at_once() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-leaked").await;
        let leaked = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");
        let rotated = fixture
            .repository
            .rotate_key(realm_id, DAY)
            .await
            .expect("rotate");
        fixture
            .repository
            .get_key_set(realm_id)
            .await
            .expect("warm the cache");

        let revoked = fixture
            .repository
            .revoke_key(realm_id, leaked.id)
            .await
            .expect("revoke")
            .expect("the key exists");

        let set = fixture
            .repository
            .get_key_set(realm_id)
            .await
            .expect("key set");

        assert_eq!(revoked.status, SigningKeyStatus::Revoked);
        assert_eq!(published(&set), vec![rotated.kid]);
        assert!(
            set.verification_key(Some(&leaked.id.to_string()), Utc::now())
                .is_none()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_the_active_key_installs_a_new_one() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-compromised").await;
        let leaked = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        fixture
            .repository
            .revoke_key(realm_id, leaked.id)
            .await
            .expect("revoke")
            .expect("the key exists");

        let active = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("active key");
        let set = fixture
            .repository
            .get_key_set(realm_id)
            .await
            .expect("key set");

        assert_ne!(active.id, leaked.id);
        assert_eq!(published(&set), vec![active.id]);
        assert_eq!(active_count(&fixture.pool, realm_id).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_twice_or_an_unknown_key_changes_nothing() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-idempotent").await;
        let other_realm = insert_realm(&fixture.pool, "tenant-other").await;
        let foreign = fixture
            .repository
            .get_or_generate_key(other_realm)
            .await
            .expect("foreign key");
        let key = fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        let first = fixture
            .repository
            .revoke_key(realm_id, key.id)
            .await
            .expect("revoke");
        let second = fixture
            .repository
            .revoke_key(realm_id, key.id)
            .await
            .expect("revoke again");
        let unknown = fixture
            .repository
            .revoke_key(realm_id, Uuid::new_v4())
            .await
            .expect("revoke unknown");
        let cross_realm = fixture
            .repository
            .revoke_key(realm_id, foreign.id)
            .await
            .expect("revoke a key of another realm");

        assert_eq!(
            first.map(|key| key.revoked_at),
            second.map(|key| key.revoked_at)
        );
        assert!(unknown.is_none());
        assert!(cross_realm.is_none());
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn concurrent_rotations_leave_exactly_one_active_key() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-racing").await;
        fixture
            .repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        let rotations = (0..4).map(|_| {
            let repository = fixture.repository.clone();
            tokio::spawn(async move { repository.rotate_key(realm_id, DAY).await })
        });
        for rotation in rotations {
            rotation
                .await
                .expect("the rotation task ran")
                .expect("every rotation succeeds");
        }

        assert_eq!(active_count(&fixture.pool, realm_id).await, 1);
        assert_eq!(count_keys(&fixture.pool, realm_id).await, 5);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn another_replica_drops_its_key_set_when_a_change_is_announced() {
        let fixture = setup().await;
        let realm_id = insert_realm(&fixture.pool, "tenant-replicated").await;
        let (elsewhere, _feed) = TokenState::new(Duration::from_secs(30));
        let replica = PostgresKeyStoreRepository::new(fixture.repository.db.clone())
            .announcing_changes(Arc::clone(&elsewhere));
        let before = replica
            .get_or_generate_key(realm_id)
            .await
            .expect("replica loads the key");

        let rotated = fixture
            .repository
            .rotate_key(realm_id, DAY)
            .await
            .expect("rotate");
        let stale = replica
            .get_or_generate_key(realm_id)
            .await
            .expect("replica still trusts its copy");
        elsewhere.changed_elsewhere();
        let fresh = replica
            .get_or_generate_key(realm_id)
            .await
            .expect("replica reloads");

        assert_eq!(stale.id, before.id);
        assert_eq!(fresh.id, rotated.kid);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_rotated_key_is_sealed_like_the_first_one() {
        let fixture = setup().await;
        let repository = sealing(fixture.repository.db.clone());
        let realm_id = insert_realm(&fixture.pool, "tenant-sealed-rotation").await;
        repository
            .get_or_generate_key(realm_id)
            .await
            .expect("first key");

        let rotated = repository.rotate_key(realm_id, DAY).await.expect("rotate");
        let stored: String = sqlx::query_scalar("SELECT private_key FROM jwt_keys WHERE id = $1")
            .bind(rotated.kid)
            .fetch_one(&fixture.pool)
            .await
            .expect("read the rotated key");

        assert!(stored.starts_with("fk1:kid-a:"), "{stored}");
        assert_eq!(
            repository
                .get_or_generate_key(realm_id)
                .await
                .expect("active key")
                .id,
            rotated.kid
        );
    }
}
