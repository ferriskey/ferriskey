use std::sync::Arc;

use crate::{
    domain::credential::{
        entities::{error::CredentialError, model::Credential},
        ports::CredentialRepository,
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
        secret: String,
        credential: String,
        label: String,
    ) -> Result<Credential, CredentialError> {
        let credential = Credential::new(
            String::new(),
            credential_type,
            user_id,
            label,
            secret,
            credential,
        );

        sqlx::query!(
            "INSERT INTO credentials (id, salt, credential_type, user_id, user_label, secret_data, credential_data) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            credential.id,
            credential.salt,
            credential.credential_type,
            credential.user_id,
            credential.user_label,
            credential.secret_data,
            credential.credential_data,
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
        let credential = sqlx::query_as!(
            Credential,
            "SELECT * FROM credentials WHERE user_id = $1 AND credential_type = 'password'",
            user_id
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map_err(|_| CredentialError::GetPasswordCredentialError)?;

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
