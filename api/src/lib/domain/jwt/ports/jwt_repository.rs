use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};

pub trait JwtRepository: Clone + Send + Sync + 'static {
    fn get_realm_key(&self, realm: &str) -> impl Future<Output = Result<String, JwtError>> + Send;
    fn generate_jwt_token(
        &self,
        claims: &JwtClaim,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;
}
