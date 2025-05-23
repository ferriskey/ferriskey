use crate::domain::crypto::entities::hash_result::HashResult;
use crate::domain::crypto::ports::{
    crypto_service::CryptoService, hasher_repository::HasherRepository,
};
use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;

pub type DefaultCryptoService = CryptoServiceImpl<Argon2HasherRepository>;

#[derive(Debug, Clone)]
pub struct CryptoServiceImpl<H>
where
    H: HasherRepository,
{
    pub hasher_repository: H,
}

impl<H> CryptoServiceImpl<H>
where
    H: HasherRepository,
{
    pub fn new(hasher_repository: H) -> Self {
        Self { hasher_repository }
    }
}

impl<H> CryptoService for CryptoServiceImpl<H>
where
    H: HasherRepository,
{
    async fn hash_password(&self, password: &str) -> Result<HashResult, anyhow::Error> {
        self.hasher_repository.hash_password(password).await
    }

    async fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        credential_data: &crate::domain::credential::entities::model::CredentialData,
        salt: &str,
    ) -> Result<bool, anyhow::Error> {
        self.hasher_repository
            .verify_password(password, secret_data, credential_data, salt)
            .await
    }
}
