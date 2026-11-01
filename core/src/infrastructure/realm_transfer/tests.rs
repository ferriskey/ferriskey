use chrono::Utc;
use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sqlx::{Executor as _, PgPool};
use uuid::Uuid;

use super::postgres_archive::PostgresRealmArchiveRepository;
use super::tables::{NOT_EXPORTED, spec_of};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::realm_transfer::bundle::{BundleHeader, BundleRealm, BundleRow};
use crate::domain::realm_transfer::ports::{RealmArchiveRepository, RowSink, RowSource};
use crate::infrastructure::realm::repositories::smtp_config_postgres_repository::SMTP_PASSWORD_PURPOSE;
use crate::infrastructure::repositories::credential_repository::OTP_SECRET_PURPOSE;
use crate::infrastructure::repositories::keystore_repository::PRIVATE_KEY_PURPOSE;

const RING_A: &str = "ring-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
const RING_B: &str = "ring-b:ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=";
const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIE\n-----END PRIVATE KEY-----";
const OTP_SEED: &str = "JBSWY3DPEHPK3PXP";
const PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA";

struct Instance {
    pool: PgPool,
    cipher: SecretCipher,
    archive: PostgresRealmArchiveRepository,
}

async fn instance(ring: &str) -> Instance {
    let base = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string());
    let schema = format!("realm_transfer_test_{}", Uuid::new_v4().simple());
    PgPool::connect(&base)
        .await
        .expect("connect admin pool")
        .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{schema}""#)))
        .await
        .expect("create schema");

    let separator = if base.contains('?') { '&' } else { '?' };
    let pool = PgPool::connect(&format!("{base}{separator}options=-c search_path={schema}"))
        .await
        .expect("connect schema pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");

    let cipher = SecretCipher::from_config(ring).expect("key ring");
    Instance {
        archive: PostgresRealmArchiveRepository::new(pool.clone(), cipher.clone()),
        pool,
        cipher,
    }
}

#[derive(Default)]
struct Collected(Vec<BundleRow>);

impl RowSink for Collected {
    async fn accept(&mut self, row: BundleRow) -> Result<(), CoreError> {
        self.0.push(row);
        Ok(())
    }
}

struct Feed(std::vec::IntoIter<BundleRow>);

impl RowSource for Feed {
    async fn next_row(&mut self) -> Result<Option<BundleRow>, CoreError> {
        Ok(self.0.next())
    }
}

struct Seeded {
    realm: BundleRealm,
    key_id: Uuid,
    retired_key_id: Uuid,
    user_id: Uuid,
    otp_id: Uuid,
    password_id: Uuid,
    role_id: Uuid,
    child_group_id: Uuid,
}

async fn exec(pool: &PgPool, sql: &str, ids: &[Uuid]) {
    let mut query = sqlx::query(sql);
    for id in ids {
        query = query.bind(*id);
    }
    query
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"));
}

async fn seed(source: &Instance, name: &str) -> Seeded {
    let pool = &source.pool;
    let realm_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();
    let retired_key_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let otp_id = Uuid::now_v7();
    let password_id = Uuid::now_v7();
    let role_id = Uuid::now_v7();
    let org_id = Uuid::now_v7();
    let parent_group_id = Uuid::now_v7();
    let child_group_id = Uuid::now_v7();

    sqlx::query(
        "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
    )
    .bind(realm_id)
    .bind(name)
    .execute(pool)
    .await
    .expect("realm");
    exec(
        pool,
        "INSERT INTO realm_settings (id, realm_id, updated_at) VALUES ($1, $2, NOW())",
        &[Uuid::now_v7(), realm_id],
    )
    .await;

    for (id, retired) in [(retired_key_id, true), (key_id, false)] {
        let sealed = source
            .cipher
            .seal(PRIVATE_KEY_PURPOSE, &id.to_string(), PRIVATE_KEY)
            .unwrap();
        sqlx::query(
            "INSERT INTO jwt_keys (id, realm_id, private_key, public_key, created_at, rotated_at, retire_at) \
             VALUES ($1, $2, $3, 'public', NOW(), \
                     CASE WHEN $4 THEN NOW() END, CASE WHEN $4 THEN NOW() END)",
        )
        .bind(id)
        .bind(realm_id)
        .bind(sealed)
        .bind(retired)
        .execute(pool)
        .await
        .expect("key");
    }

    let smtp_id = Uuid::now_v7();
    let smtp_password = source
        .cipher
        .seal(SMTP_PASSWORD_PURPOSE, &smtp_id.to_string(), "smtp-secret")
        .unwrap();
    sqlx::query(
        "INSERT INTO smtp_configs (id, realm_id, host, port, username, password, from_email, from_name, encryption) \
         VALUES ($1, $2, 'smtp.local', 587, 'mailer', $3, 'no-reply@acme.test', 'Acme', 'tls')",
    )
    .bind(smtp_id)
    .bind(realm_id)
    .bind(smtp_password)
    .execute(pool)
    .await
    .expect("smtp");

    exec(pool, "INSERT INTO users (id, realm_id, username, email, email_verified, enabled, created_at, updated_at) VALUES ($1, $2, 'alice', 'alice@acme.test', true, true, NOW(), NOW())", &[user_id, realm_id]).await;
    sqlx::query(
        "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, created_at, updated_at) \
         VALUES ($1, 'password', $2, $3, '{}'::jsonb, NOW(), NOW())",
    )
    .bind(password_id)
    .bind(user_id)
    .bind(PASSWORD_HASH)
    .execute(pool)
    .await
    .expect("password");
    let otp = source
        .cipher
        .seal(OTP_SECRET_PURPOSE, &otp_id.to_string(), OTP_SEED)
        .unwrap();
    sqlx::query(
        "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, created_at, updated_at) \
         VALUES ($1, 'otp', $2, $3, '{}'::jsonb, NOW(), NOW())",
    )
    .bind(otp_id)
    .bind(user_id)
    .bind(otp)
    .execute(pool)
    .await
    .expect("otp");

    exec(pool, "INSERT INTO roles (id, name, permissions, realm_id, created_at, updated_at) VALUES ($1, 'reader', 0, $2, NOW(), NOW())", &[role_id, realm_id]).await;
    exec(pool, "INSERT INTO user_role (user_id, role_id, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())", &[user_id, role_id]).await;
    exec(pool, "INSERT INTO organizations (id, realm_id, name, alias, enabled, created_at, updated_at) VALUES ($1, $2, 'Acme', 'acme', true, NOW(), NOW())", &[org_id, realm_id]).await;
    exec(pool, "INSERT INTO organization_groups (id, organization_id, parent_group_id, name, created_at, updated_at) VALUES ($1, $2, NULL, 'parent', NOW(), NOW())", &[parent_group_id, org_id]).await;
    exec(pool, "INSERT INTO organization_groups (id, organization_id, parent_group_id, name, created_at, updated_at) VALUES ($1, $2, $3, 'child', NOW(), NOW())", &[child_group_id, org_id, parent_group_id]).await;
    exec(pool, "INSERT INTO user_sessions (id, user_id, realm_id, created_at, expires_at, persistent, authenticated_at) VALUES ($1, $2, $3, NOW(), NOW() + interval '1 day', true, NOW())", &[Uuid::now_v7(), user_id, realm_id]).await;

    Seeded {
        realm: BundleRealm {
            id: realm_id,
            name: name.to_string(),
        },
        key_id,
        retired_key_id,
        user_id,
        otp_id,
        password_id,
        role_id,
        child_group_id,
    }
}

fn header(realm: &BundleRealm) -> BundleHeader {
    BundleHeader::new(realm.clone(), 0, Utc::now())
}

async fn export(source: &Instance, realm: &BundleRealm) -> Vec<BundleRow> {
    let mut rows = Collected::default();
    source
        .archive
        .export_rows(RealmId::new(realm.id), &mut rows)
        .await
        .expect("export");
    rows.0
}

async fn import(
    target: &Instance,
    realm: &BundleRealm,
    rows: Vec<BundleRow>,
) -> Result<crate::domain::realm_transfer::value_objects::ImportReport, CoreError> {
    target
        .archive
        .import_rows(&header(realm), &mut Feed(rows.into_iter()))
        .await
}

async fn stored(pool: &PgPool, sql: &str, id: Uuid) -> String {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"))
}

async fn count(pool: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .expect("count")
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn every_table_of_the_schema_is_either_exported_or_left_out_on_purpose() {
    let instance = instance(RING_A).await;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables \
         WHERE table_schema = current_schema() AND table_type = 'BASE TABLE'",
    )
    .fetch_all(&instance.pool)
    .await
    .expect("tables");

    let unclassified: Vec<&String> = tables
        .iter()
        .filter(|table| spec_of(table).is_none() && !NOT_EXPORTED.contains(&table.as_str()))
        .collect();
    assert!(
        unclassified.is_empty(),
        "classify {unclassified:?} in realm_transfer/tables.rs"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_realm_moves_with_its_ids_and_its_secrets_sealed_again_for_the_target() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;

    let rows = export(&source, &seeded.realm).await;
    let key = rows
        .iter()
        .find(|row| row.table == "jwt_keys" && row.row["id"] == seeded.key_id.to_string())
        .expect("the active key is exported");
    assert_eq!(key.row["private_key"], PRIVATE_KEY);
    assert!(
        rows.iter()
            .all(|row| row.table != "user_role" || !row.row.contains_key("id"))
    );

    let report = import(&target, &seeded.realm, rows).await.expect("import");
    assert!(report.created);
    assert_eq!(report.rows["jwt_keys"], 2);
    assert!(
        report.ignored_columns.is_empty(),
        "{:?}",
        report.ignored_columns
    );

    for kid in [seeded.key_id, seeded.retired_key_id] {
        let sealed = stored(
            &target.pool,
            "SELECT private_key FROM jwt_keys WHERE id = $1",
            kid,
        )
        .await;
        assert!(sealed.starts_with("fk1:ring-b:"), "{sealed}");
        assert_eq!(
            target
                .cipher
                .open(PRIVATE_KEY_PURPOSE, &kid.to_string(), &sealed)
                .unwrap(),
            PRIVATE_KEY
        );
    }
    let otp = stored(
        &target.pool,
        "SELECT secret_data FROM credentials WHERE id = $1",
        seeded.otp_id,
    )
    .await;
    assert_eq!(
        target
            .cipher
            .open(OTP_SECRET_PURPOSE, &seeded.otp_id.to_string(), &otp)
            .unwrap(),
        OTP_SEED
    );
    assert_eq!(
        stored(
            &target.pool,
            "SELECT secret_data FROM credentials WHERE id = $1",
            seeded.password_id
        )
        .await,
        PASSWORD_HASH
    );
    let user_role: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_role WHERE user_id = $1 AND role_id = $2")
            .bind(seeded.user_id)
            .bind(seeded.role_id)
            .fetch_one(&target.pool)
            .await
            .unwrap();
    assert_eq!(user_role, 1);
    assert_eq!(
        stored(
            &target.pool,
            "SELECT name FROM organization_groups WHERE id = $1",
            seeded.child_group_id
        )
        .await,
        "child"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn importing_the_same_bundle_twice_leaves_the_same_rows() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let rows = export(&source, &seeded.realm).await;

    let first = import(&target, &seeded.realm, rows.clone())
        .await
        .expect("first import");
    let counts: Vec<i64> = futures::future::join_all(
        [
            "users",
            "credentials",
            "user_role",
            "jwt_keys",
            "user_sessions",
        ]
        .map(|t| count(&target.pool, t)),
    )
    .await;
    let second = import(&target, &seeded.realm, rows)
        .await
        .expect("second import");
    let again: Vec<i64> = futures::future::join_all(
        [
            "users",
            "credentials",
            "user_role",
            "jwt_keys",
            "user_sessions",
        ]
        .map(|t| count(&target.pool, t)),
    )
    .await;

    assert!(first.created);
    assert!(!second.created);
    assert_eq!(first.rows, second.rows);
    assert_eq!(counts, again);
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_row_pointing_into_another_realm_fails_the_whole_import() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let elsewhere = seed(&target, "elsewhere").await;
    let mut rows = export(&source, &seeded.realm).await;

    let credential = rows
        .iter_mut()
        .find(|row| row.table == "credentials")
        .expect("a credential");
    credential
        .row
        .insert("user_id".to_string(), elsewhere.user_id.to_string().into());

    let refused = import(&target, &seeded.realm, rows).await;

    assert!(
        matches!(refused, Err(CoreError::RealmBundleConflict(_))),
        "{refused:?}"
    );
    let realms: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM realms WHERE id = $1")
        .bind(seeded.realm.id)
        .fetch_one(&target.pool)
        .await
        .unwrap();
    assert_eq!(realms, 0, "nothing of the refused import is kept");
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_row_with_the_id_of_another_realms_row_is_refused() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let elsewhere = seed(&target, "elsewhere").await;
    let mut rows = export(&source, &seeded.realm).await;

    let role = rows
        .iter_mut()
        .find(|row| row.table == "roles")
        .expect("a role");
    role.row
        .insert("id".to_string(), elsewhere.role_id.to_string().into());
    rows.retain(|row| row.table != "user_role");

    let refused = import(&target, &seeded.realm, rows).await;

    assert!(
        matches!(refused, Err(CoreError::RealmBundleConflict(_))),
        "{refused:?}"
    );
    assert_eq!(
        stored(
            &target.pool,
            "SELECT realm_id::text FROM roles WHERE id = $1",
            elsewhere.role_id
        )
        .await,
        elsewhere.realm.id.to_string()
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_bundle_whose_keys_were_removed_is_refused() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let mut rows = export(&source, &seeded.realm).await;
    rows.retain(|row| row.table != "jwt_keys");

    assert!(matches!(
        import(&target, &seeded.realm, rows).await,
        Err(CoreError::RealmBundleWithoutKeys)
    ));
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_username_taken_by_another_user_stops_a_reimport() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let rows = export(&source, &seeded.realm).await;
    import(&target, &seeded.realm, rows.clone())
        .await
        .expect("first import");

    sqlx::query("UPDATE users SET username = 'renamed' WHERE id = $1")
        .bind(seeded.user_id)
        .execute(&target.pool)
        .await
        .unwrap();
    exec(&target.pool, "INSERT INTO users (id, realm_id, username, email_verified, enabled, created_at, updated_at) VALUES ($1, $2, 'alice', false, true, NOW(), NOW())", &[Uuid::now_v7(), seeded.realm.id]).await;

    assert!(matches!(
        import(&target, &seeded.realm, rows).await,
        Err(CoreError::RealmBundleConflict(_))
    ));
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_sealed_value_in_a_column_the_export_does_not_know_stops_the_export() {
    let source = instance(RING_A).await;
    let seeded = seed(&source, "acme").await;
    let sealed = source
        .cipher
        .seal("users.firstname", "x", "secret")
        .unwrap();
    sqlx::query("UPDATE users SET firstname = $1 WHERE id = $2")
        .bind(sealed)
        .bind(seeded.user_id)
        .execute(&source.pool)
        .await
        .unwrap();

    let mut rows = Collected::default();
    let refused = source
        .archive
        .export_rows(RealmId::new(seeded.realm.id), &mut rows)
        .await;

    assert!(
        matches!(refused, Err(CoreError::InvalidRealmBundle(_))),
        "{refused:?}"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn an_unknown_column_from_an_older_schema_is_ignored_and_reported() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    let mut rows = export(&source, &seeded.realm).await;
    rows.iter_mut()
        .filter(|row| row.table == "users")
        .for_each(|row| {
            row.row.insert("nickname".to_string(), "al".into());
        });

    let report = import(&target, &seeded.realm, rows).await.expect("import");

    assert_eq!(report.ignored_columns, vec!["users.nickname".to_string()]);
}

#[tokio::test]
#[ignore = "requires PostgreSQL"]
async fn a_realm_name_taken_by_another_realm_is_refused() {
    let source = instance(RING_A).await;
    let target = instance(RING_B).await;
    let seeded = seed(&source, "acme").await;
    seed(&target, "acme").await;
    let rows = export(&source, &seeded.realm).await;

    assert!(matches!(
        import(&target, &seeded.realm, rows).await,
        Err(CoreError::RealmBundleConflict(_))
    ));
}
