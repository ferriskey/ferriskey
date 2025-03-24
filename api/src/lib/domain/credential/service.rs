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
        user_id: uuid::Uuid,
        password: String,
        label: String,
    ) -> Result<Credential, CredentialError> {
        let (secret, credential) = self
            .hasher_repository
            .hash_password(&password)
            .await
            .map_err(|e| CredentialError::HashPasswordError(e.to_string()))?;

        let cred = self
            .credential_repository
            .create_credential(user_id, "password".to_string(), secret, credential, label)
            .await?;

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
        todo!("Implement this")
    }
}
