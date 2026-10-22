use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        realm::{
            entities::{RealmId, SmtpConfig},
            ports::SmtpConfigRepository,
        },
    },
    entity::smtp_configs::{ActiveModel, Entity as SmtpConfigEntity},
};

pub const SMTP_PASSWORD_PURPOSE: &str = "smtp_configs.password";

#[derive(Debug, Clone)]
pub struct PostgresSmtpConfigRepository {
    pub db: DatabaseConnection,
    cipher: SecretCipher,
}

impl PostgresSmtpConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_cipher(db, SecretCipher::disabled())
    }

    pub fn with_cipher(db: DatabaseConnection, cipher: SecretCipher) -> Self {
        Self { db, cipher }
    }

    fn seal_password(&self, id: Uuid, password: &str) -> Result<String, CoreError> {
        self.cipher
            .seal(SMTP_PASSWORD_PURPOSE, &id.to_string(), password)
            .map_err(|error| CoreError::Configuration(error.to_string()))
    }

    fn config_of(
        &self,
        model: crate::entity::smtp_configs::Model,
    ) -> Result<SmtpConfig, CoreError> {
        let id = model.id;
        let mut config = SmtpConfig::from(model);
        config.password = self
            .cipher
            .open(SMTP_PASSWORD_PURPOSE, &id.to_string(), &config.password)
            .map_err(|error| CoreError::Configuration(error.to_string()))?;

        Ok(config)
    }
}

impl SmtpConfigRepository for PostgresSmtpConfigRepository {
    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Option<SmtpConfig>, CoreError> {
        let result = SmtpConfigEntity::find()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?
            .map(|model| self.config_of(model))
            .transpose()?;

        Ok(result)
    }

    async fn upsert(&self, config: &SmtpConfig) -> Result<SmtpConfig, CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        let existing = SmtpConfigEntity::find()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq(config.realm_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let model = if let Some(existing) = existing {
            let existing_id = existing.id;
            let mut active: ActiveModel = existing.into();
            active.host = Set(config.host.clone());
            active.port = Set(config.port as i32);
            active.username = Set(config.username.clone());
            active.password = Set(self.seal_password(existing_id, &config.password)?);
            active.from_email = Set(config.from_email.clone());
            active.from_name = Set(config.from_name.clone());
            active.encryption = Set(config.encryption.as_str().to_string());
            active.updated_at = Set(now);

            active
                .update(&self.db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
        } else {
            let id = crate::domain::common::generate_uuid_v7();
            let active = ActiveModel {
                id: Set(id),
                realm_id: Set(config.realm_id),
                host: Set(config.host.clone()),
                port: Set(config.port as i32),
                username: Set(config.username.clone()),
                password: Set(self.seal_password(id, &config.password)?),
                from_email: Set(config.from_email.clone()),
                from_name: Set(config.from_name.clone()),
                encryption: Set(config.encryption.as_str().to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active
                .insert(&self.db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
        };

        self.config_of(model)
    }

    async fn delete_by_realm_id(&self, realm_id: RealmId) -> Result<(), CoreError> {
        SmtpConfigEntity::delete_many()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::entities::SmtpEncryption;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    const SEALING_KEY: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

    async fn setup(prefix: &str) -> (sea_orm::DatabaseConnection, sqlx::PgPool) {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("{prefix}_{}", Uuid::new_v4().simple());

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

        (db, pool)
    }

    async fn insert_realm(pool: &sqlx::PgPool) -> Uuid {
        let realm_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("realm-{}", realm_id.simple()))
        .execute(pool)
        .await
        .expect("insert realm");
        realm_id
    }

    fn config_for(realm_id: Uuid, password: &str) -> SmtpConfig {
        SmtpConfig {
            id: Uuid::new_v4(),
            realm_id,
            host: "smtp.example.com".to_string(),
            port: 587,
            username: "mailer".to_string(),
            password: password.to_string(),
            from_email: "no-reply@example.com".to_string(),
            from_name: "Example".to_string(),
            encryption: SmtpEncryption::StartTls,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn sealing(db: DatabaseConnection) -> PostgresSmtpConfigRepository {
        PostgresSmtpConfigRepository::with_cipher(
            db,
            SecretCipher::from_config(SEALING_KEY).expect("a valid key ring"),
        )
    }

    async fn stored_password(pool: &sqlx::PgPool, realm_id: Uuid) -> String {
        sqlx::query_scalar("SELECT password FROM smtp_configs WHERE realm_id = $1")
            .bind(realm_id)
            .fetch_one(pool)
            .await
            .expect("read the stored password")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_password_is_sealed_at_rest_on_insert_and_on_update() {
        let (db, pool) = setup("smtp_config_test").await;
        let realm_id = insert_realm(&pool).await;
        let repository = sealing(db);

        let saved = repository
            .upsert(&config_for(realm_id, "first-password"))
            .await
            .expect("insert the configuration");
        assert_eq!(saved.password, "first-password");
        let stored = stored_password(&pool, realm_id).await;
        assert!(stored.starts_with("fk1:kid-a:"), "{stored}");
        assert!(!stored.contains("first-password"));

        let updated = repository
            .upsert(&config_for(realm_id, "second-password"))
            .await
            .expect("update the configuration");
        assert_eq!(updated.password, "second-password");
        assert!(
            stored_password(&pool, realm_id)
                .await
                .starts_with("fk1:kid-a:")
        );

        let read = repository
            .get_by_realm_id(RealmId::from(realm_id))
            .await
            .expect("read the configuration")
            .expect("a configuration");
        assert_eq!(read.password, "second-password");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_plaintext_password_is_still_read_when_a_cipher_is_configured() {
        let (db, pool) = setup("smtp_config_test").await;
        let realm_id = insert_realm(&pool).await;
        PostgresSmtpConfigRepository::new(db.clone())
            .upsert(&config_for(realm_id, "legacy-password"))
            .await
            .expect("insert a legacy configuration");

        let read = sealing(db)
            .get_by_realm_id(RealmId::from(realm_id))
            .await
            .expect("read the configuration")
            .expect("a configuration");

        assert_eq!(read.password, "legacy-password");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sealed_password_is_refused_by_an_instance_without_the_key() {
        let (db, pool) = setup("smtp_config_test").await;
        let realm_id = insert_realm(&pool).await;
        sealing(db.clone())
            .upsert(&config_for(realm_id, "secret-password"))
            .await
            .expect("insert the configuration");

        let result = PostgresSmtpConfigRepository::new(db)
            .get_by_realm_id(RealmId::from(realm_id))
            .await;

        assert!(result.is_err(), "{result:?}");
    }
}
