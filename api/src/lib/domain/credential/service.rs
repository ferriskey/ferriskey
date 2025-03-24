use std::sync::Arc;

use crate::domain::crypto::ports::HasherRepository;

use super::{
    entities::{error::CredentialError, model::Credential},
    ports::{CredentialRepository, CredentialService},
};

#[derive(Debug, Clone)]
pub struct CredentialServiceImpl<H, C>
where
    H: HasherRepository,
    C: CredentialRepository,
{
    hasher_repository: Arc<H>,
    credential_repository: C,
}

impl<H, C> CredentialServiceImpl<H, C>
where
    H: HasherRepository,
    C: CredentialRepository,
{
    pub fn new(hasher_repository: Arc<H>, credential_repository: C) -> Self {
        Self {
            hasher_repository,
            credential_repository,
        }
    }
}

impl<H, C> CredentialService for CredentialServiceImpl<H, C>
where
    H: HasherRepository,
    C: CredentialRepository,
{
    async fn create_password_credential(
        &self,
        _user_id: uuid::Uuid,
        _password: String,
        _label: String,
    ) -> Result<Credential, CredentialError> {
        todo!("Implement this")
    }

    async fn reset_password(
        &self,
        user_id: uuid::Uuid,
        password: String,
    ) -> Result<(), CredentialError> {
        let (secret, salt) = self
            .hasher_repository
            .hash_password(&password)
            .await
            .map_err(|e| CredentialError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(
                user_id,
                "password".to_string(),
                secret,
                salt,
                String::from("My password"),
            )
            .await?;

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

        let is_valid = self
            .hasher_repository
            .verify_password(
                &password,
                &credential.secret_data,
                &credential.credential_data,
            )
            .await
            .map_err(|e| CredentialError::VerifyPasswordError(e.to_string()))?;

        Ok(is_valid)
    }
}
