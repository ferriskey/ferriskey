use chrono::{TimeZone, Utc};
use entity::credentials::{ActiveModel, Entity as CredentialEntity};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use tracing::error;

use crate::domain::{
    common::{generate_timestamp, generate_uuid_v7},
    credential::{
        entities::{Credential, CredentialData, CredentialError},
        ports::CredentialRepository,
    },
    crypto::entities::HashResult,
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
            .filter(entity::credentials::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|_| CredentialError::GetUserCredentialsError)?
            .into_iter()
            .map(Credential::from)
            .collect();

        Ok(credentials)
    }

    async fn delete_by_id(&self, credential_id: uuid::Uuid) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(entity::credentials::Column::Id.eq(credential_id))
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
        };

        let model = payload
            .insert(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(model.into())
    }
}
