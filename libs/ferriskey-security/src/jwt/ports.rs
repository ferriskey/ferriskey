use chrono::{DateTime, Utc};
use ferriskey_domain::realm::{Realm, RealmId};
use uuid::Uuid;

use crate::{
    SecurityError,
    jwt::entities::{Jwt, JwtClaim, JwtKeyPair, RefreshToken},
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
        relam_id: RealmId,
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

#[cfg_attr(test, mockall::automock)]
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

pub trait KeyStoreRepository: Send + Sync {
    fn get_or_generate_key(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<JwtKeyPair, SecurityError>> + Send;
}
