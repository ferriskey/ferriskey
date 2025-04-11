use std::sync::Arc;

use crate::domain::credential::entities::error::CredentialError;
use crate::domain::credential::entities::model::Credential;
use crate::domain::credential::ports::credential_repository::CredentialRepository;
use crate::domain::credential::ports::credential_service::CredentialService;
use crate::domain::crypto::ports::crypto_service::CryptoService;
use crate::domain::crypto::services::crypto_service::DefaultCryptoService;
use crate::infrastructure::repositories::credential_repository::PostgresCredentialRepository;

pub type DefaultCredentialService = CredentialServiceImpl<PostgresCredentialRepository>;

#[derive(Debug, Clone)]
pub struct CredentialServiceImpl<C>
where
    C: CredentialRepository,
{
    credential_repository: C,
    crypto_service: Arc<DefaultCryptoService>,
}

impl<C> CredentialServiceImpl<C>
where
    C: CredentialRepository,
{
    pub fn new(credential_repository: C, crypto_service: Arc<DefaultCryptoService>) -> Self {
        Self {
            credential_repository,
            crypto_service,
        }
    }
}

impl<C> CredentialService for CredentialServiceImpl<C>
where
    C: CredentialRepository,
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
        _user_id: uuid::Uuid,
        _password: String,
    ) -> Result<(), CredentialError> {
        unimplemented!("Reset password")
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
}
