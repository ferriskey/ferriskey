use chrono::{DateTime, Utc};
use ferriskey_domain::realm::{Realm, RealmId};
use uuid::Uuid;

use crate::{
    SecurityError,
    jwt::entities::{AccessToken, Jwt, JwtClaim, JwtKeyPair, RefreshToken},
};

pub trait JwtService: Send + Sync {
    fn generate_token(
        &self,
        claims: JwtClaim,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Jwt, SecurityError>> + Send;
    fn verify_token(
        &self,
        token: String,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<JwtClaim, SecurityError>> + Send;
    fn verify_refresh_token(
        &self,
        token: String,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<JwtClaim, SecurityError>> + Send;

    fn retrieve_realm_rsa_keys(
        &self,
        realm: &Realm,
    ) -> impl Future<Output = Result<JwtKeyPair, SecurityError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RefreshTokenRepository: Send + Sync {
    fn create(
        &self,
        jti: Uuid,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<RefreshToken, SecurityError>> + Send;
    fn get_by_jti(
        &self,
        jti: Uuid,
    ) -> impl Future<Output = Result<RefreshToken, SecurityError>> + Send;
    fn delete(&self, jti: Uuid) -> impl Future<Output = Result<(), SecurityError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AccessTokenRepository: Send + Sync {
    fn create(
        &self,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: RealmId,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
    ) -> impl Future<Output = Result<AccessToken, SecurityError>> + Send;

    fn get_by_token_hash(
        &self,
        token_hash: String,
    ) -> impl Future<Output = Result<Option<AccessToken>, SecurityError>> + Send;

    fn revoke_by_token_hash(
        &self,
        token_hash: String,
    ) -> impl Future<Output = Result<(), SecurityError>> + Send;
}

pub trait KeyStoreRepository: Send + Sync {
    fn get_or_generate_key(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<JwtKeyPair, SecurityError>> + Send;
}
