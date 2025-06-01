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
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;

    fn verify_token(
        &self,
        token: String,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;

    fn verify_refresh_token(
        &self,
        token: String,
    ) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;

    fn generate_refresh_token(
        &self,
        claims: JwtClaim,
    ) -> impl Future<Output = Result<Jwt, JwtError>> + Send;

    fn retrieve_realm_rsa_keys(
        &self,
        realm: &Realm,
    ) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
}
