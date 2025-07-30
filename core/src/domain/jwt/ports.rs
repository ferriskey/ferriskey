use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    jwt::entities::{Jwt, JwtClaim, JwtError, JwtKeyPair, RefreshToken},
    realm::entities::Realm,
};

pub trait JwtService: Clone + Send + Sync + 'static {
    fn generate_token(
        &self,
        claims: JwtClaim,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;

    fn verify_token(
        &self,
        token: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;

    fn verify_refresh_token(
        &self,
        token: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;

    fn retrieve_realm_rsa_keys(
        &self,
        realm: &Realm,
    ) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
}

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

pub trait KeyStoreRepository: Clone + Send + Sync + 'static {
    fn get_or_generate_key(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
}
