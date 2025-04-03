use async_trait::async_trait;

use super::{
    entities::{Jwt, JwtClaims, JwtError},
    ports::{JwtRepository, JwtService},
};

pub struct JwtServiceImpl {
    pub repository: Box<dyn JwtRepository>,
}

impl JwtServiceImpl {
    pub fn new(repository: Box<dyn JwtRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl JwtService for JwtServiceImpl {
    async fn generate_token(&self, claims: JwtClaims) -> Result<Jwt, JwtError> {
        self.repository.generate_jwt_token(&claims).await
    }
}
