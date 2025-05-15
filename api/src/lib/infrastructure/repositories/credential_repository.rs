use chrono::{TimeZone, Utc};
use entity::credentials::{ActiveModel, Entity as CredentialEntity};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use sqlx::{Executor, PgPool};

use crate::domain::{
    credential::{
        entities::{
            error::CredentialError,
            model::{Credential, CredentialData},
        },
        ports::credential_repository::CredentialRepository,
    },
    crypto::entities::hash_result::HashResult,
    utils::{generate_timestamp, generate_uuid_v7},
};

impl From<entity::credentials::Model> for Credential {
    fn from(model: entity::credentials::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        let credential_data = serde_json::from_value(model.credential_data)
            .map_err(|_| CredentialError::GetPasswordCredentialError)
            .unwrap_or(CredentialData {
                hash_iterations: 0,
                algorithm: "default".to_string(),
            });

        Self {
            id: model.id,
            salt: model.salt,
            credential_type: model.credential_type,
            user_id: model.user_id,
            user_label: model.user_label,
            secret_data: model.secret_data,
            credential_data,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresCredentialRepository {
    pub pool: PgPool,
    pub db: DatabaseConnection,
}

impl PostgresCredentialRepository {
    pub fn new(pool: PgPool, db: DatabaseConnection) -> Self {
        Self { pool, db }
    }
}

impl CredentialRepository for PostgresCredentialRepository {
    async fn create_credential(
        &self,
        user_id: uuid::Uuid,
        credential_type: String,
        hash_result: HashResult,
        label: String,
    ) -> Result<Credential, CredentialError> {
        let (now, _) = generate_timestamp();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            salt: Set(Some(hash_result.salt)),
            credential_type: Set(credential_type),
            user_id: Set(user_id),
            user_label: Set(Some(label)),
            secret_data: Set(hash_result.hash),
            credential_data: Set(serde_json::to_value(&hash_result.credential_data)
                .map_err(|_| CredentialError::CreateCredentialError)?),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        };

        let t = payload
            .insert(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(t.into())
    }

    async fn get_password_credential(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Credential, CredentialError> {
        let credential = CredentialEntity::find()
            .filter(entity::credentials::Column::UserId.eq(user_id))
            .filter(entity::credentials::Column::CredentialType.eq("password"))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::GetPasswordCredentialError)?
            .map(Credential::from);

        let credential = credential.ok_or(CredentialError::GetPasswordCredentialError)?;

        Ok(credential)
    }

    async fn delete_password_credential(&self, user_id: uuid::Uuid) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(entity::credentials::Column::UserId.eq(user_id))
            .filter(entity::credentials::Column::CredentialType.eq("password"))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::DeletePasswordCredentialError)?
            .ok_or(CredentialError::DeletePasswordCredentialError)?;

        credential
            .delete(&self.db)
            .await
            .map_err(|_| CredentialError::DeletePasswordCredentialError)?;

        Ok(())
    }
}
