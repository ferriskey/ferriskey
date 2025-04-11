use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};

pub trait JwtService: Clone + Send + Sync + 'static {
    fn generate_token(
        &self,
        claims: JwtClaim,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;
}
