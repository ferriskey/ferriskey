use async_trait::async_trait;

use super::{entities::{Jwt, JwtClaims, JwtError}, ports::{JwtRepository, JwtService}};

pub struct JwtServiceImpl {
    pub repository: Box<dyn JwtRepository + Send + Sync + 'static>
}

impl JwtServiceImpl {
    pub fn new(repository: impl JwtRepository + Send + Sync + 'static) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }
}

#[async_trait]
impl JwtService for JwtServiceImpl {
    async fn generate_token(&self, claims: JwtClaims) -> Result<Jwt, JwtError> {
        self.repository.generate_jwt_token(&claims).await
    }
}
