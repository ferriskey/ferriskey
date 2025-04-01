use std::future::Future;
use super::ports::{CryptoService, HasherRepository};

#[derive(Debug, Clone)]
pub struct CryptoServiceImpl<H> 
where
    H: HasherRepository
{
    pub hasher_repository: H
}

impl<H> CryptoServiceImpl<H> 
where
    H: HasherRepository
{
    pub fn new(hasher_repository: H) -> Self {
        Self { hasher_repository }
    }
}


impl<H> CryptoService for CryptoServiceImpl<H>
where
    H: HasherRepository
{
    async fn hash_password(&self, password: &str) -> Result<(String, String), anyhow::Error> {
        self.hasher_repository.hash_password(password).await
    }
}