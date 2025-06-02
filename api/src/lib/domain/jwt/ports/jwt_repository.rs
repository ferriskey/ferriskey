use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::jwt::entities::{jwt_error::JwtError, refresh_token::RefreshToken};

pub trait RefreshTokenRepository: Clone + Send + Sync + 'static {
    fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<RefreshToken, JwtError>> + Send;
    fn get_by_jti(&self, jti: Uuid) -> impl Future<Output = Result<RefreshToken, JwtError>> + Send;
    fn delete(&self, jti: Uuid) -> impl Future<Output = Result<(), JwtError>> + Send;
}
