use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sea_orm::{Database, DatabaseConnection};
use sqlx::Executor as _;
use uuid::Uuid;

use crate::domain::realm::entities::{Realm, RealmId, RealmScope};

pub const SEALING_KEY: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

pub fn sealing_cipher() -> SecretCipher {
    SecretCipher::from_config(SEALING_KEY).expect("a valid key ring")
}

pub struct TestSchema {
    pub db: DatabaseConnection,
    pub pool: sqlx::PgPool,
    pub realm: RealmScope,
}

pub async fn migrated_schema(prefix: &str) -> TestSchema {
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string());
    let schema = format!("{prefix}_{}", Uuid::new_v4().simple());

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
    let name = format!("realm-{}", realm_id.simple());
    sqlx::query(
        "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
    )
    .bind(realm_id)
    .bind(&name)
    .execute(&pool)
    .await
    .expect("insert realm");

    let mut realm = Realm::new(name);
    realm.id = RealmId::from(realm_id);

    TestSchema {
        db,
        pool,
        realm: RealmScope::from_realm(realm),
    }
}

pub async fn stored(pool: &sqlx::PgPool, table: &str, value: &str, id: Uuid) -> Option<String> {
    sqlx::query_scalar(&format!("SELECT {value} FROM {table} WHERE id = $1"))
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("read the stored value")
}
