use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use async_trait::async_trait;

#[async_trait]
pub trait JwtService: Send + Sync {
    async fn generate_token(&self, claims: JwtClaim) -> Result<Jwt, JwtError>;
}
