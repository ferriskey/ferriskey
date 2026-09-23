use crate::{
    domain::credential::entities::CredentialType,
    entity::credentials::{ActiveModel, Entity as CredentialEntity},
};
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, sea_query::Expr,
};
use serde_json::Value;
use tracing::error;
use webauthn_rs::prelude::{AuthenticationResult, CredentialID, Passkey};

use crate::domain::{
    common::{generate_timestamp, generate_uuid_v7},
    credential::{
        entities::{Credential, CredentialData, CredentialError},
        ports::CredentialRepository,
    },
    crypto::HashResult,
    realm::entities::Scoped,
    user::entities::User,
};

impl From<crate::entity::credentials::Model> for Credential {
    fn from(model: crate::entity::credentials::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        let credential_data = serde_json::from_value(model.credential_data)
            .map_err(|_| CredentialError::GetPasswordCredentialError)
            .unwrap_or(CredentialData::Hash {
                hash_iterations: 0,
                algorithm: "default".to_string(),
            });

        let webauthn_credential_id = model.webauthn_credential_id.map(CredentialID::from);

        Self {
            id: model.id,
            salt: model.salt,
            credential_type: model.credential_type.into(),
            user_id: model.user_id,
            user_label: model.user_label,
            secret_data: model.secret_data,
            credential_data,
            temporary: model.temporary.unwrap_or(false),
            created_at,
            updated_at,
            webauthn_credential_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresCredentialRepository {
    pub db: DatabaseConnection,
}

impl PostgresCredentialRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl CredentialRepository for PostgresCredentialRepository {
    async fn create_credential(
        &self,
        user_id: uuid::Uuid,
        credential_type: String,
        hash_result: HashResult,
        label: String,
        temporary: bool,
    ) -> Result<Credential, CredentialError> {
        let (now, _) = generate_timestamp();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            salt: Set(Some(hash_result.salt).filter(|salt| !salt.is_empty())),
            credential_type: Set(credential_type),
            user_id: Set(user_id),
            user_label: Set(Some(label)),
            secret_data: Set(hash_result.hash),
            credential_data: Set(serde_json::to_value(CredentialData::new_hash(
                hash_result.hash_iterations,
                hash_result.algorithm.clone(),
            ))
            .map_err(|e| {
                error!("Error serializing credential_data for user {user_id}: {e:?}");
                CredentialError::CreateCredentialError
            })?),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
            temporary: Set(Some(temporary)), // Assuming credentials are not temporary by default
            webauthn_credential_id: Set(None),
        };

        let t = payload.insert(&self.db).await.map_err(|e| {
            error!("Error creating credential for user {user_id}: {e:?}");
            CredentialError::CreateCredentialError
        })?;

        Ok(t.into())
    }

    async fn get_password_credential(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Credential, CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(crate::entity::credentials::Column::CredentialType.eq("password"))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::GetPasswordCredentialError)?
            .map(Credential::from);

        let credential = credential.ok_or(CredentialError::GetPasswordCredentialError)?;

        Ok(credential)
    }

    async fn update_password_credential(
        &self,
        user_id: uuid::Uuid,
        expected_secret_data: &str,
        hash_result: HashResult,
    ) -> Result<(), CredentialError> {
        let (now, _) = generate_timestamp();

        let credential_data = serde_json::to_value(CredentialData::new_hash(
            hash_result.hash_iterations,
            hash_result.algorithm,
        ))
        .map_err(|e| {
            error!("Error serializing credential_data for user {user_id}: {e:?}");
            CredentialError::UpdateCredentialError
        })?;

        let result = CredentialEntity::update_many()
            .col_expr(
                crate::entity::credentials::Column::SecretData,
                Expr::value(hash_result.hash),
            )
            .col_expr(
                crate::entity::credentials::Column::Salt,
                Expr::value(hash_result.salt),
            )
            .col_expr(
                crate::entity::credentials::Column::CredentialData,
                Expr::value(credential_data),
            )
            .col_expr(
                crate::entity::credentials::Column::UpdatedAt,
                Expr::value(now.naive_utc()),
            )
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(
                crate::entity::credentials::Column::CredentialType
                    .eq(CredentialType::Password.as_str()),
            )
            .filter(crate::entity::credentials::Column::SecretData.eq(expected_secret_data))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating password credential for user {user_id}: {e:?}");
                CredentialError::UpdateCredentialError
            })?;

        if result.rows_affected == 0 {
            return Err(CredentialError::UpdateCredentialError);
        }

        Ok(())
    }

    async fn has_password_credential(&self, user_id: uuid::Uuid) -> Result<bool, CredentialError> {
        CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(
                crate::entity::credentials::Column::CredentialType
                    .eq(CredentialType::Password.as_str()),
            )
            .one(&self.db)
            .await
            .map(|credential| credential.is_some())
            .map_err(|e| {
                error!("Error checking password credential for user {user_id}: {e:?}");
                CredentialError::GetPasswordCredentialError
            })
    }

    async fn delete_password_credential(&self, user_id: uuid::Uuid) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(
                crate::entity::credentials::Column::CredentialType
                    .eq(CredentialType::Password.as_str()),
            )
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error fetching password credential: {:?}", e);
                CredentialError::DeletePasswordCredentialError
            })?
            .ok_or(CredentialError::DeletePasswordCredentialError)?;

        credential.delete(&self.db).await.map_err(|e| {
            error!("Error deleting password credential: {:?}", e);
            CredentialError::DeletePasswordCredentialError
        })?;

        Ok(())
    }

    async fn get_credentials_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Credential>, CredentialError> {
        let credentials = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|_| CredentialError::GetUserCredentialsError)?
            .into_iter()
            .map(Credential::from)
            .collect();

        Ok(credentials)
    }

    async fn delete_by_id(
        &self,
        user: &Scoped<User>,
        credential_id: uuid::Uuid,
    ) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::Id.eq(credential_id))
            .filter(crate::entity::credentials::Column::UserId.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::DeleteCredentialError)?
            .ok_or(CredentialError::DeleteCredentialError)?;

        credential
            .delete(&self.db)
            .await
            .map_err(|_| CredentialError::DeleteCredentialError)?;

        Ok(())
    }

    async fn create_custom_credential(
        &self,
        user_id: uuid::Uuid,
        credential_type: String, // "TOTP", "WEBAUTHN", etc.
        secret_data: String,     // base32 pour TOTP
        label: Option<String>,
        credential_data: serde_json::Value,
    ) -> Result<Credential, CredentialError> {
        let (now, _) = generate_timestamp();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            salt: Set(None),
            credential_type: Set(credential_type),
            user_id: Set(user_id),
            user_label: Set(label),
            secret_data: Set(secret_data),
            credential_data: Set(credential_data),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
            temporary: Set(Some(false)), // Assuming custom credentials are not temporary
            webauthn_credential_id: Set(None),
        };

        let model = payload
            .insert(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(model.into())
    }

    async fn create_recovery_code_credentials(
        &self,
        user_id: uuid::Uuid,
        hashes: Vec<HashResult>,
    ) -> Result<(), CredentialError> {
        let (now, _) = generate_timestamp();

        let credential_data = hashes
            .iter()
            .map(|h| {
                serde_json::to_value(CredentialData::new_hash(
                    h.hash_iterations,
                    h.algorithm.clone(),
                ))
                .map_err(|_| CredentialError::CreateCredentialError)
            })
            .collect::<Result<Vec<Value>, CredentialError>>()?;

        let models = hashes
            .into_iter()
            .zip(credential_data)
            .map(|(h, cred_data)| ActiveModel {
                id: Set(generate_uuid_v7()),
                salt: Set(Some(h.salt)),
                credential_type: Set("recovery-code".to_string()),
                user_id: Set(user_id),
                user_label: Set(None),
                secret_data: Set(h.hash),
                credential_data: Set(cred_data),
                created_at: Set(now.naive_utc()),
                updated_at: Set(now.naive_utc()),
                temporary: Set(Some(false)),
                webauthn_credential_id: Set(None),
            });

        let _ = CredentialEntity::insert_many(models)
            .exec(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(())
    }

    async fn create_webauthn_credential(
        &self,
        user_id: uuid::Uuid,
        webauthn_credential: Passkey,
    ) -> Result<Credential, CredentialError> {
        let (now, _) = generate_timestamp();

        let credential_id = Vec::from(webauthn_credential.cred_id().to_owned());
        let credential_data = CredentialData::new_webauthn(webauthn_credential);

        let credential_data = serde_json::to_value(credential_data)
            .map_err(|_| CredentialError::CreateCredentialError)?;

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            salt: Set(None),
            credential_type: Set(CredentialType::WebAuthnPublicKeyCredential.to_string()),
            user_id: Set(user_id),
            user_label: Set(None),
            secret_data: Set("".to_string()),
            credential_data: Set(credential_data),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
            temporary: Set(Some(false)),
            webauthn_credential_id: Set(Some(credential_id)),
        };

        let model = payload
            .insert(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(model.into())
    }

    async fn get_webauthn_public_key_credentials(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Credential>, CredentialError> {
        let credentials = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(
                crate::entity::credentials::Column::CredentialType
                    .eq(CredentialType::WebAuthnPublicKeyCredential.as_str()),
            )
            .all(&self.db)
            .await
            .map_err(|_| CredentialError::GetUserCredentialsError)?
            .into_iter()
            .map(Credential::from)
            .collect();

        Ok(credentials)
    }

    async fn get_webauthn_credential_by_credential_id_and_user(
        &self,
        credential_id: &[u8],
        user: &Scoped<User>,
    ) -> Result<Option<Credential>, CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::WebauthnCredentialId.eq(credential_id))
            .filter(crate::entity::credentials::Column::UserId.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::GetUserCredentialsError)?
            .map(Credential::from);

        Ok(credential)
    }

    async fn update_webauthn_credential(
        &self,
        user: &Scoped<User>,
        auth_result: &AuthenticationResult,
    ) -> Result<bool, CredentialError> {
        let credential_id: &[u8] = auth_result.cred_id().as_slice();

        let credential_model = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::WebauthnCredentialId.eq(credential_id))
            .filter(crate::entity::credentials::Column::UserId.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::GetUserCredentialsError)?
            .ok_or(CredentialError::GetUserCredentialsError)?;

        let credential: Credential = credential_model.clone().into();

        let update_res;
        let updated_data = match credential.credential_data {
            CredentialData::WebAuthn { credential } => {
                let mut passkey = Passkey::from(*credential);

                update_res = passkey
                    .update_credential(auth_result)
                    .ok_or(CredentialError::UpdateCredentialError)?;

                CredentialData::WebAuthn {
                    credential: Box::new(passkey.into()),
                }
            }
            _ => return Err(CredentialError::UnexpectedCredentialData),
        };

        let updated_data = serde_json::to_value(updated_data)
            .map_err(|_| CredentialError::UpdateCredentialError)?;

        let mut active_model = credential_model.into_active_model();
        active_model.credential_data = Set(updated_data);
        active_model
            .update(&self.db)
            .await
            .map_err(|_| CredentialError::UpdateCredentialError)?;

        Ok(update_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    struct Fixture {
        repository: PostgresCredentialRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("credential_repository_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresCredentialRepository::new(db),
            pool,
        }
    }

    async fn insert_user(pool: &sqlx::PgPool) -> Uuid {
        let realm_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("realm-{}", realm_id.simple()))
        .execute(pool)
        .await
        .expect("insert realm");

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) \
             VALUES ($1, $2, 'imported', '', '', 'imported@example.com')",
        )
        .bind(user_id)
        .bind(realm_id)
        .execute(pool)
        .await
        .expect("insert user");

        user_id
    }

    async fn insert_bcrypt_password(pool: &sqlx::PgPool, user_id: Uuid, temporary: bool) {
        sqlx::query(
            "INSERT INTO credentials \
             (id, salt, credential_type, user_id, secret_data, credential_data, temporary) \
             VALUES ($1, NULL, 'password', $2, '$2a$10$legacy', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(serde_json::json!({ "hash_iterations": 10, "algorithm": "bcrypt" }))
        .bind(temporary)
        .execute(pool)
        .await
        .expect("insert bcrypt credential");
    }

    fn argon2id_hash() -> HashResult {
        HashResult::new(
            "$argon2id$v=19$m=7168,t=5,p=1$new".to_string(),
            "new_salt".to_string(),
            5,
            "argon2id".to_string(),
        )
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_password_credential_replaces_the_hash_in_place() {
        let fixture = setup().await;
        let user_id = insert_user(&fixture.pool).await;
        insert_bcrypt_password(&fixture.pool, user_id, true).await;
        let before = fixture
            .repository
            .get_password_credential(user_id)
            .await
            .expect("bcrypt credential");

        fixture
            .repository
            .update_password_credential(user_id, "$2a$10$legacy", argon2id_hash())
            .await
            .expect("update password credential");

        let after = fixture
            .repository
            .get_password_credential(user_id)
            .await
            .expect("argon2id credential");

        assert_eq!(after.id, before.id);
        assert_eq!(after.secret_data, "$argon2id$v=19$m=7168,t=5,p=1$new");
        assert_eq!(after.salt.as_deref(), Some("new_salt"));
        assert!(after.temporary);
        assert!(matches!(
            after.credential_data,
            CredentialData::Hash { hash_iterations: 5, ref algorithm } if algorithm == "argon2id"
        ));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn create_credential_stores_an_empty_salt_as_null() {
        let fixture = setup().await;
        let user_id = insert_user(&fixture.pool).await;

        fixture
            .repository
            .create_credential(
                user_id,
                "password".to_string(),
                HashResult::new(
                    "$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy".to_string(),
                    String::new(),
                    10,
                    "bcrypt".to_string(),
                ),
                String::new(),
                false,
            )
            .await
            .expect("create bcrypt credential");

        let credential = fixture
            .repository
            .get_password_credential(user_id)
            .await
            .expect("bcrypt credential");

        assert_eq!(credential.salt, None);
        assert!(matches!(
            credential.credential_data,
            CredentialData::Hash { hash_iterations: 10, ref algorithm } if algorithm == "bcrypt"
        ));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_password_credential_keeps_a_hash_changed_since_verification() {
        let fixture = setup().await;
        let user_id = insert_user(&fixture.pool).await;
        insert_bcrypt_password(&fixture.pool, user_id, false).await;
        sqlx::query("UPDATE credentials SET secret_data = '$argon2id$reset' WHERE user_id = $1")
            .bind(user_id)
            .execute(&fixture.pool)
            .await
            .expect("simulate a concurrent reset");

        let result = fixture
            .repository
            .update_password_credential(user_id, "$2a$10$legacy", argon2id_hash())
            .await;

        assert!(matches!(
            result,
            Err(CredentialError::UpdateCredentialError)
        ));
        let credential = fixture
            .repository
            .get_password_credential(user_id)
            .await
            .expect("password credential");
        assert_eq!(credential.secret_data, "$argon2id$reset");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_password_credential_fails_without_a_password_credential() {
        let fixture = setup().await;
        let user_id = insert_user(&fixture.pool).await;

        let result = fixture
            .repository
            .update_password_credential(user_id, "$2a$10$legacy", argon2id_hash())
            .await;

        assert!(matches!(
            result,
            Err(CredentialError::UpdateCredentialError)
        ));
    }
}
