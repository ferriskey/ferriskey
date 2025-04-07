use async_trait::async_trait;

use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use crate::domain::jwt::ports::{jwt_repository::JwtRepository, jwt_service::JwtService};

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
    async fn generate_token(&self, claims: JwtClaim) -> Result<Jwt, JwtError> {
        self.repository.generate_jwt_token(&claims).await
    }
}
