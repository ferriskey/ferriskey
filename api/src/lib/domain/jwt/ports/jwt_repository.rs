use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use async_trait::async_trait;

#[async_trait]
pub trait JwtRepository: Send + Sync {
    async fn get_realm_key(&self, realm: &str) -> Result<String, JwtError>;
    async fn generate_jwt_token(&self, claims: &JwtClaim) -> Result<Jwt, JwtError>;
}
