use std::sync::Arc;

use crate::domain::crypto::ports::CryptoService;

use super::{
    entities::{error::CredentialError, model::Credential},
    ports::{CredentialRepository, CredentialService},
};

#[derive(Debug, Clone)]
pub struct CredentialServiceImpl<C, CS>
where
    C: CredentialRepository,
    CS: CryptoService,
{
    credential_repository: C,
    crypto_service: Arc<CS>,
}

impl<C, CS> CredentialServiceImpl<C, CS>
where
    C: CredentialRepository,
    CS: CryptoService,
{
    pub fn new(credential_repository: C, crypto_service: Arc<CS>) -> Self {
        Self {
            credential_repository,
            crypto_service,
        }
    }
}

impl<C, CS> CredentialService for CredentialServiceImpl<C, CS>
where
    C: CredentialRepository,
    CS: CryptoService,
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
