use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use tracing::{error, info};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::infrastructure::abyss::federation::repository::BIND_PASSWORD_PURPOSE;
use crate::infrastructure::client::repositories::client_postgres_repository::CLIENT_SECRET_PURPOSE;
use crate::infrastructure::identity_provider::repositories::identity_provider_postgres_repository::IDENTITY_PROVIDER_SECRET_PURPOSE;
use crate::infrastructure::realm::repositories::smtp_config_postgres_repository::SMTP_PASSWORD_PURPOSE;
use crate::infrastructure::repositories::credential_repository::OTP_SECRET_PURPOSE;
use crate::infrastructure::repositories::keystore_repository::PRIVATE_KEY_PURPOSE;
use crate::infrastructure::trident::repositories::otp_enrollment_repository::ENROLLMENT_SECRET_PURPOSE;
use crate::infrastructure::webhook::repositories::webhook_repository::WEBHOOK_SECRET_PURPOSE;

const BATCH_SIZE: i64 = 200;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SealedCounts {
    pub private_keys: u64,
    pub otp_credentials: u64,
    pub otp_enrollments: u64,
    pub smtp_passwords: u64,
    pub client_secrets: u64,
    pub identity_provider_secrets: u64,
    pub ldap_bind_passwords: u64,
    pub webhook_secrets: u64,
}

struct SecretColumn {
    table: &'static str,
    value: &'static str,
    assign: &'static str,
    filter: &'static str,
    purpose: &'static str,
}

const SECRET_COLUMNS: [SecretColumn; 8] = [
    SecretColumn {
        table: "jwt_keys",
        value: "private_key",
        assign: "private_key = $1",
        filter: "TRUE",
        purpose: PRIVATE_KEY_PURPOSE,
    },
    SecretColumn {
        table: "credentials",
        value: "secret_data",
        assign: "secret_data = $1",
        filter: "credential_type = 'otp'",
        purpose: OTP_SECRET_PURPOSE,
    },
    SecretColumn {
        table: "otp_enrollments",
        value: "secret",
        assign: "secret = $1",
        filter: "TRUE",
        purpose: ENROLLMENT_SECRET_PURPOSE,
    },
    SecretColumn {
        table: "smtp_configs",
        value: "password",
        assign: "password = $1",
        filter: "TRUE",
        purpose: SMTP_PASSWORD_PURPOSE,
    },
    SecretColumn {
        table: "clients",
        value: "secret",
        assign: "secret = $1",
        filter: "secret IS NOT NULL",
        purpose: CLIENT_SECRET_PURPOSE,
    },
    SecretColumn {
        table: "identity_providers",
        value: "config #>> '{client_secret}'",
        assign: "config = jsonb_set(config, '{client_secret}', to_jsonb($1::text))",
        filter: "jsonb_typeof(config #> '{client_secret}') = 'string'",
        purpose: IDENTITY_PROVIDER_SECRET_PURPOSE,
    },
    SecretColumn {
        table: "user_federation_providers",
        value: "config #>> '{bind,bind_password_encrypted}'",
        assign: "config = jsonb_set(config, '{bind,bind_password_encrypted}', to_jsonb($1::text))",
        filter: "jsonb_typeof(config #> '{bind,bind_password_encrypted}') = 'string'",
        purpose: BIND_PASSWORD_PURPOSE,
    },
    SecretColumn {
        table: "webhooks",
        value: "secret",
        assign: "secret = $1",
        filter: "TRUE",
        purpose: WEBHOOK_SECRET_PURPOSE,
    },
];

fn database_error(error: sea_orm::DbErr) -> CoreError {
    CoreError::Database(error.to_string())
}

async fn seal_column(
    db: &DatabaseConnection,
    cipher: &SecretCipher,
    active: &str,
    spec: &SecretColumn,
) -> Result<u64, CoreError> {
    let pattern = format!("fk1:{}:%", active.replace('_', "\\_"));
    let select = format!(
        "SELECT id, {value} AS value FROM {table} \
         WHERE {filter} AND {value} NOT LIKE $1 AND id > $2 ORDER BY id LIMIT {BATCH_SIZE}",
        value = spec.value,
        table = spec.table,
        filter = spec.filter,
    );
    let update = format!(
        "UPDATE {table} SET {assign} WHERE id = $2 AND {value} = $3",
        assign = spec.assign,
        value = spec.value,
        table = spec.table,
    );

    let mut sealed = 0;
    let mut after = Uuid::nil();

    loop {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &select,
                [pattern.clone().into(), after.into()],
            ))
            .await
            .map_err(database_error)?;

        let batch = rows.len();

        for row in rows {
            let id: Uuid = row.try_get("", "id").map_err(database_error)?;
            let value: String = row.try_get("", "value").map_err(database_error)?;
            after = id;

            let resealed = match cipher
                .open(spec.purpose, &id.to_string(), &value)
                .and_then(|plain| cipher.seal(spec.purpose, &id.to_string(), &plain))
            {
                Ok(resealed) => resealed,
                Err(error) => {
                    error!(table = spec.table, %id, %error, "a secret could not be sealed");
                    continue;
                }
            };

            let result = db
                .execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &update,
                    [resealed.into(), id.into(), value.into()],
                ))
                .await
                .map_err(database_error)?;
            sealed += result.rows_affected();
        }

        if (batch as i64) < BATCH_SIZE {
            return Ok(sealed);
        }
    }
}

pub async fn seal_existing_secrets(
    db: &DatabaseConnection,
    cipher: &SecretCipher,
) -> Result<SealedCounts, CoreError> {
    let Some(active) = cipher.active_key_id() else {
        return Ok(SealedCounts::default());
    };

    let counts = SealedCounts {
        private_keys: seal_column(db, cipher, active, &SECRET_COLUMNS[0]).await?,
        otp_credentials: seal_column(db, cipher, active, &SECRET_COLUMNS[1]).await?,
        otp_enrollments: seal_column(db, cipher, active, &SECRET_COLUMNS[2]).await?,
        smtp_passwords: seal_column(db, cipher, active, &SECRET_COLUMNS[3]).await?,
        client_secrets: seal_column(db, cipher, active, &SECRET_COLUMNS[4]).await?,
        identity_provider_secrets: seal_column(db, cipher, active, &SECRET_COLUMNS[5]).await?,
        ldap_bind_passwords: seal_column(db, cipher, active, &SECRET_COLUMNS[6]).await?,
        webhook_secrets: seal_column(db, cipher, active, &SECRET_COLUMNS[7]).await?,
    };

    info!(
        ?counts,
        key = active,
        "sealed the stored secrets that were not under the active key"
    );

    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    const KEY_A: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
    const KEY_B: &str = "kid-b:ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=";

    fn cipher(spec: &str) -> SecretCipher {
        SecretCipher::from_config(spec).expect("a valid key ring")
    }

    struct Seeded {
        db: sea_orm::DatabaseConnection,
        pool: sqlx::PgPool,
        key_id: Uuid,
        credential_id: Uuid,
        enrollment_id: Uuid,
        smtp_id: Uuid,
        client_id: Uuid,
        public_client_id: Uuid,
        identity_provider_id: Uuid,
        federation_id: Uuid,
        webhook_id: Uuid,
    }

    async fn seed() -> Seeded {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("secrets_backfill_test_{}", Uuid::new_v4().simple());

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

        let realm_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let (key_id, credential_id, enrollment_id, smtp_id) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let (client_id, public_client_id, identity_provider_id, federation_id, webhook_id) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        for statement in [
            sqlx::query("INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, 'sweep', NOW(), NOW())")
                .bind(realm_id),
            sqlx::query("INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, 'u', '', '', 'u@example.com')")
                .bind(user_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO jwt_keys (id, realm_id, private_key, public_key, created_at) VALUES ($1, $2, 'PRIVATE-PEM', 'PUBLIC-PEM', NOW())")
                .bind(key_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data) VALUES ($1, 'otp', $2, 'JBSWY3DP', '{}'::jsonb)")
                .bind(credential_id)
                .bind(user_id),
            sqlx::query("INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data) VALUES ($1, 'password', $2, '$argon2id$hash', '{}'::jsonb)")
                .bind(Uuid::new_v4())
                .bind(user_id),
            sqlx::query("INSERT INTO otp_enrollments (id, user_id, secret, expires_at) VALUES ($1, $2, 'KRSXG5CT', NOW() + interval '5 minutes')")
                .bind(enrollment_id)
                .bind(user_id),
            sqlx::query("INSERT INTO smtp_configs (id, realm_id, host, port, username, password, from_email, from_name, encryption, created_at, updated_at) VALUES ($1, $2, 'smtp.example.com', 587, 'm', 'smtp-secret', 'a@example.com', 'A', 'starttls', NOW(), NOW())")
                .bind(smtp_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO clients (id, realm_id, name, client_id, secret, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) VALUES ($1, $2, 'c', 'confidential', 'client-secret', TRUE, 'openid-connect', FALSE, TRUE, 'confidential', NOW(), NOW())")
                .bind(client_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) VALUES ($1, $2, 'p', 'public', TRUE, 'openid-connect', TRUE, FALSE, 'public', NOW(), NOW())")
                .bind(public_client_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO identity_providers (id, realm_id, alias, provider_id, enabled, config, created_at, updated_at) VALUES ($1, $2, 'google', 'oidc', TRUE, '{\"client_id\": \"broker\", \"client_secret\": \"idp-secret\"}'::jsonb, NOW(), NOW())")
                .bind(identity_provider_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO user_federation_providers (id, realm_id, name, provider_type, config) VALUES ($1, $2, 'corp', 'Ldap', '{\"bind\": {\"bind_dn\": \"cn=admin\", \"bind_password_encrypted\": \"ldap-secret\"}}'::jsonb)")
                .bind(federation_id)
                .bind(realm_id),
            sqlx::query("INSERT INTO webhooks (id, realm_id, endpoint, headers, secret, created_at, updated_at) VALUES ($1, $2, 'https://example.test/hook', '{}'::jsonb, 'webhook-secret', NOW(), NOW())")
                .bind(webhook_id)
                .bind(realm_id),
        ] {
            statement.execute(&pool).await.expect("seed a legacy row");
        }

        Seeded {
            db,
            pool,
            key_id,
            credential_id,
            enrollment_id,
            smtp_id,
            client_id,
            public_client_id,
            identity_provider_id,
            federation_id,
            webhook_id,
        }
    }

    async fn stored(pool: &sqlx::PgPool, table: &str, column: &str, id: Uuid) -> String {
        sqlx::query_scalar(&format!("SELECT {column} FROM {table} WHERE id = $1"))
            .bind(id)
            .fetch_one(pool)
            .await
            .expect("read the stored value")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn every_legacy_secret_is_sealed_in_place() {
        let seeded = seed().await;
        let cipher = cipher(KEY_A);

        let sealed = seal_existing_secrets(&seeded.db, &cipher)
            .await
            .expect("seal the legacy secrets");

        assert_eq!(
            sealed,
            SealedCounts {
                private_keys: 1,
                otp_credentials: 1,
                otp_enrollments: 1,
                smtp_passwords: 1,
                client_secrets: 1,
                identity_provider_secrets: 1,
                ldap_bind_passwords: 1,
                webhook_secrets: 1,
            }
        );
        for (table, column, id, plain, purpose) in [
            (
                "jwt_keys",
                "private_key",
                seeded.key_id,
                "PRIVATE-PEM",
                PRIVATE_KEY_PURPOSE,
            ),
            (
                "credentials",
                "secret_data",
                seeded.credential_id,
                "JBSWY3DP",
                OTP_SECRET_PURPOSE,
            ),
            (
                "otp_enrollments",
                "secret",
                seeded.enrollment_id,
                "KRSXG5CT",
                ENROLLMENT_SECRET_PURPOSE,
            ),
            (
                "smtp_configs",
                "password",
                seeded.smtp_id,
                "smtp-secret",
                SMTP_PASSWORD_PURPOSE,
            ),
            (
                "clients",
                "secret",
                seeded.client_id,
                "client-secret",
                CLIENT_SECRET_PURPOSE,
            ),
            (
                "identity_providers",
                "config #>> '{client_secret}'",
                seeded.identity_provider_id,
                "idp-secret",
                IDENTITY_PROVIDER_SECRET_PURPOSE,
            ),
            (
                "user_federation_providers",
                "config #>> '{bind,bind_password_encrypted}'",
                seeded.federation_id,
                "ldap-secret",
                BIND_PASSWORD_PURPOSE,
            ),
            (
                "webhooks",
                "secret",
                seeded.webhook_id,
                "webhook-secret",
                WEBHOOK_SECRET_PURPOSE,
            ),
        ] {
            let value = stored(&seeded.pool, table, column, id).await;
            assert!(value.starts_with("fk1:kid-a:"), "{table}: {value}");
            assert_eq!(
                cipher.open(purpose, &id.to_string(), &value).unwrap(),
                plain
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn password_credentials_are_left_alone() {
        let seeded = seed().await;

        seal_existing_secrets(&seeded.db, &cipher(KEY_A))
            .await
            .expect("seal the legacy secrets");

        let hashes: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM credentials WHERE credential_type = 'password' AND secret_data = '$argon2id$hash'",
        )
        .fetch_one(&seeded.pool)
        .await
        .expect("count the password hashes");
        assert_eq!(hashes, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn only_the_secret_inside_a_json_config_is_rewritten() {
        let seeded = seed().await;

        seal_existing_secrets(&seeded.db, &cipher(KEY_A))
            .await
            .expect("seal the legacy secrets");

        let public_secret: Option<String> =
            sqlx::query_scalar("SELECT secret FROM clients WHERE id = $1")
                .bind(seeded.public_client_id)
                .fetch_one(&seeded.pool)
                .await
                .expect("read the public client");
        assert_eq!(public_secret, None);
        assert_eq!(
            stored(
                &seeded.pool,
                "identity_providers",
                "config ->> 'client_id'",
                seeded.identity_provider_id
            )
            .await,
            "broker"
        );
        assert_eq!(
            stored(
                &seeded.pool,
                "user_federation_providers",
                "config #>> '{bind,bind_dn}'",
                seeded.federation_id
            )
            .await,
            "cn=admin"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_second_sweep_has_nothing_left_to_seal() {
        let seeded = seed().await;
        let cipher = cipher(KEY_A);
        seal_existing_secrets(&seeded.db, &cipher)
            .await
            .expect("first sweep");

        let again = seal_existing_secrets(&seeded.db, &cipher)
            .await
            .expect("second sweep");

        assert_eq!(again, SealedCounts::default());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_new_active_key_reseals_what_the_old_key_sealed() {
        let seeded = seed().await;
        seal_existing_secrets(&seeded.db, &cipher(KEY_A))
            .await
            .expect("seal with the first key");
        let rotated = cipher(&format!("{KEY_B},{KEY_A}"));

        let resealed = seal_existing_secrets(&seeded.db, &rotated)
            .await
            .expect("reseal with the new key");

        assert_eq!(
            resealed,
            SealedCounts {
                private_keys: 1,
                otp_credentials: 1,
                otp_enrollments: 1,
                smtp_passwords: 1,
                client_secrets: 1,
                identity_provider_secrets: 1,
                ldap_bind_passwords: 1,
                webhook_secrets: 1,
            }
        );
        let value = stored(&seeded.pool, "jwt_keys", "private_key", seeded.key_id).await;
        assert!(value.starts_with("fk1:kid-b:"), "{value}");
        assert_eq!(
            rotated
                .open(PRIVATE_KEY_PURPOSE, &seeded.key_id.to_string(), &value)
                .unwrap(),
            "PRIVATE-PEM"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn without_a_cipher_nothing_is_touched() {
        let seeded = seed().await;

        let sealed = seal_existing_secrets(&seeded.db, &SecretCipher::disabled())
            .await
            .expect("sweep without a cipher");

        assert_eq!(sealed, SealedCounts::default());
        assert_eq!(
            stored(&seeded.pool, "jwt_keys", "private_key", seeded.key_id).await,
            "PRIVATE-PEM"
        );
    }
}
