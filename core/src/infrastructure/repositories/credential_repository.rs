use crate::entity::credentials::{ActiveModel, Entity as CredentialEntity};
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use serde_json::Value;
use tracing::error;

use crate::domain::{
    common::{generate_timestamp, generate_uuid_v7},
    credential::{
        entities::{Credential, CredentialData, CredentialError},
        ports::CredentialRepository,
    },
    crypto::entities::HashResult,
};

impl From<crate::entity::credentials::Model> for Credential {
    fn from(model: crate::entity::credentials::Model) -> Self {
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
            credential_type: model.credential_type.into(),
            user_id: model.user_id,
            user_label: model.user_label,
            secret_data: model.secret_data,
            credential_data,
            temporary: model.temporary.unwrap_or(false),
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
        temporary: bool,
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
            temporary: Set(Some(temporary)), // Assuming credentials are not temporary by default
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
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(crate::entity::credentials::Column::CredentialType.eq("password"))
            .one(&self.db)
            .await
            .map_err(|_| CredentialError::GetPasswordCredentialError)?
            .map(Credential::from);

        let credential = credential.ok_or(CredentialError::GetPasswordCredentialError)?;

        Ok(credential)
    }

    async fn delete_password_credential(&self, user_id: uuid::Uuid) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::UserId.eq(user_id))
            .filter(crate::entity::credentials::Column::CredentialType.eq("password"))
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

    async fn delete_by_id(&self, credential_id: uuid::Uuid) -> Result<(), CredentialError> {
        let credential = CredentialEntity::find()
            .filter(crate::entity::credentials::Column::Id.eq(credential_id))
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
                serde_json::to_value(&h.credential_data)
                    .map_err(|_| CredentialError::CreateCredentialError)
            })
            .collect::<Result<Vec<Value>, CredentialError>>()?;

        let models = hashes
            .into_iter()
            .zip(credential_data.into_iter())
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
            });

        let _ = CredentialEntity::insert_many(models)
            .exec(&self.db)
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(())
    }
}
