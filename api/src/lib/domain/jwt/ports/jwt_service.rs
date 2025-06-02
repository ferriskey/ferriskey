use uuid::Uuid;

use crate::domain::{
    jwt::entities::{
        jwt::{Jwt, JwtKeyPair},
        jwt_claim::JwtClaim,
        jwt_error::JwtError,
    },
    realm::entities::realm::Realm,
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
