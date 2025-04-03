use async_trait::async_trait;

use super::entities::{Jwt, JwtClaims, JwtError};


#[async_trait]
pub trait JwtService: Send + Sync {
    async fn generate_token(&self, claims: JwtClaims) -> Result<Jwt, JwtError>;
}

#[async_trait]
pub trait JwtRepository: Send + Sync {
    async fn get_realm_key(&self, realm: &str) -> Result<String, JwtError>;
    async fn generate_jwt_token(&self, claims: &JwtClaims) -> Result<Jwt, JwtError>;
}
