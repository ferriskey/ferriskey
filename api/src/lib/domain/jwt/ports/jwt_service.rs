use uuid::Uuid;

use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};

pub trait JwtService: Clone + Send + Sync + 'static {
    fn generate_token(
        &self,
        claims: JwtClaim,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;

    fn verify_token(
        &self,
        token: String,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;

    fn generate_refresh_token(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;
}
