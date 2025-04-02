use std::sync::Arc;

use crate::{
    domain::{
        credential::{
            entities::{
                error::CredentialError,
                model::{Credential, CredentialData},
            },
            ports::CredentialRepository,
        },
        crypto::ports::HashResult,
    },
    infrastructure::db::postgres::Postgres,
};

#[derive(Debug, Clone)]
pub struct PostgresCredentialRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresCredentialRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
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
        let credential = Credential::new(
            hash_result.salt,
            credential_type,
            user_id,
            label,
            hash_result.hash,
            hash_result.credential_data,
        );

        let credential_data = serde_json::to_value(&credential.credential_data)
            .map_err(|_| CredentialError::CreateCredentialError)?;

        sqlx::query!(
            "INSERT INTO credentials (id, salt, credential_type, user_id, user_label, secret_data, credential_data) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            credential.id,
            credential.salt,
            credential.credential_type,
            credential.user_id,
            credential.user_label,
            credential.secret_data,
            credential_data,
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(credential)
    }

    async fn get_password_credential(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Credential, CredentialError> {
        let row = sqlx::query!(
            "SELECT * FROM credentials WHERE user_id = $1 AND credential_type = 'password'",
            user_id
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map_err(|_| CredentialError::GetPasswordCredentialError)?;

        let credential_data: CredentialData = serde_json::from_value(row.credential_data)
            .map_err(|_| CredentialError::GetPasswordCredentialError)?;

        let credential = Credential {
            id: row.id,
            user_id: row.user_id,
            credential_type: row.credential_type,
            salt: row.salt,
            secret_data: row.secret_data,
            user_label: row.user_label,
            credential_data,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };
        Ok(credential)
    }

    async fn delete_password_credential(&self, user_id: uuid::Uuid) -> Result<(), CredentialError> {
        sqlx::query!(
            "DELETE FROM credentials WHERE user_id = $1 AND credential_type = 'password'",
            user_id
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| CredentialError::DeletePasswordCredentialError)?;

        Ok(())
    }
}
