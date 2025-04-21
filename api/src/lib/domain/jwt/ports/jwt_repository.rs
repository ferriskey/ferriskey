use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::jwt::entities::{
    jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError, refresh_token::RefreshToken,
};

pub trait JwtRepository: Clone + Send + Sync + 'static {
    fn get_realm_key(&self, realm: &str) -> impl Future<Output = Result<String, JwtError>> + Send;
    fn generate_jwt_token(
        &self,
        claims: &JwtClaim,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;

    fn verify_token(
        &self,
        token: String,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;
}

pub trait RefreshTokenRepository: Clone + Send + Sync + 'static {
    fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<RefreshToken, JwtError>> + Send;
    fn delete(&self, jti: Uuid) -> impl Future<Output = Result<(), JwtError>> + Send;
}
