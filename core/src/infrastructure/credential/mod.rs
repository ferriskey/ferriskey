use crate::domain::credential::entities::{Credential, CredentialError};
use crate::domain::credential::ports::CredentialRepository;
use crate::domain::crypto::entities::HashResult;
use crate::infrastructure::repositories::credential_repository::PostgresCredentialRepository;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub enum CredentialRepoAny {
    Postgres(PostgresCredentialRepository),
}

impl CredentialRepository for CredentialRepoAny {
    async fn create_credential(
        &self,
        user_id: Uuid,
        credential_type: String,
        hash_result: HashResult,
        label: String,
    ) -> Result<Credential, CredentialError> {
        todo!()
    }

    async fn get_password_credential(
        &self,
        user_id: Uuid,
    ) -> Result<Credential, CredentialError> {
        todo!()
    }

    async fn delete_password_credential(
        &self,
        user_id: Uuid,
    ) -> Result<(), CredentialError> {
        todo!()
    }

    async fn get_credentials_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Credential>, CredentialError> {
        todo!()
    }

    async fn delete_by_id(
        &self,
        credential_id: Uuid,
    ) -> Result<(), CredentialError> {
        todo!()
    }

    async fn create_custom_credential(
        &self,
        user_id: Uuid,
        credential_type: String,
        secret_data: String,
        label: Option<String>,
        credential_data: Value,
    ) -> Result<Credential, CredentialError> {
        todo!()
    }
}
