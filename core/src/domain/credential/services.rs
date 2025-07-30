use crate::domain::crypto::ports::CryptoService;
use crate::domain::{
    credential::{
        entities::{Credential, CredentialError},
        ports::{CredentialRepository, CredentialService},
    },
    crypto::{ports::HasherRepository, services::CryptoServiceImpl},
};

#[derive(Debug, Clone)]
pub struct CredentialServiceImpl<C, H>
where
    C: CredentialRepository,
    H: HasherRepository,
{
    credential_repository: C,
    crypto_service: CryptoServiceImpl<H>,
}

impl<C, H> CredentialServiceImpl<C, H>
where
    C: CredentialRepository,
    H: HasherRepository,
{
    pub fn new(credential_repository: C, crypto_service: CryptoServiceImpl<H>) -> Self {
        Self {
            credential_repository,
            crypto_service,
        }
    }
}

impl<C, H> CredentialService for CredentialServiceImpl<C, H>
where
    C: CredentialRepository,
    H: HasherRepository,
{
    async fn create_password_credential(
        &self,
        user_id: uuid::Uuid,
        password: String,
        label: String,
    ) -> Result<Credential, CredentialError> {
        let hash = self
            .crypto_service
            .hash_password(&password)
            .await
            .map_err(|e| CredentialError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(user_id, "password".to_string(), hash, label)
            .await
    }

    async fn reset_password(
        &self,
        user_id: uuid::Uuid,
        password: String,
    ) -> Result<(), CredentialError> {
        let password_credential = self
            .credential_repository
            .get_password_credential(user_id)
            .await;

        if password_credential.is_ok() {
            self.credential_repository
                .delete_password_credential(user_id)
                .await?;
        }

        let hash_result = self
            .crypto_service
            .hash_password(&password)
            .await
            .map_err(|e| CredentialError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(user_id, "password".into(), hash_result, "".into())
            .await
            .map_err(|_| CredentialError::CreateCredentialError)?;

        Ok(())
    }

    async fn verify_password(
        &self,
        user_id: uuid::Uuid,
        password: String,
    ) -> Result<bool, CredentialError> {
        let credential = self
            .credential_repository
            .get_password_credential(user_id)
            .await?;

        let salt = credential.salt.ok_or(CredentialError::VerifyPasswordError(
            "Salt is not found".to_string(),
        ))?;

        let is_valid = self
            .crypto_service
            .verify_password(
                &password,
                &credential.secret_data,
                &credential.credential_data,
                &salt,
            )
            .await
            .map_err(|e| CredentialError::VerifyPasswordError(e.to_string()))?;

        Ok(is_valid)
    }

    async fn get_credentials_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Credential>, CredentialError> {
        self.credential_repository
            .get_credentials_by_user_id(user_id)
            .await
    }

    async fn delete_by_id(&self, credential_id: uuid::Uuid) -> Result<(), CredentialError> {
        self.credential_repository.delete_by_id(credential_id).await
    }

    async fn create_custom_credential(
        &self,
        user_id: uuid::Uuid,
        credential_type: String,
        secret_data: String,
        label: Option<String>,
        credential_data: serde_json::Value,
    ) -> Result<Credential, CredentialError> {
        self.credential_repository
            .create_custom_credential(
                user_id,
                credential_type,
                secret_data,
                label,
                credential_data,
            )
            .await
    }
}
