use crate::domain::jwt::entities::jwt::JwtKeyPair;
use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use crate::domain::jwt::ports::jwt_repository::{JwtRepository, RefreshTokenRepository};
use crate::domain::jwt::ports::jwt_service::JwtService;
use crate::domain::jwt::ports::keystore_repository::KeyStoreRepository;
use crate::domain::realm::entities::realm::Realm;
use crate::domain::realm::ports::realm_repository::RealmRepository;
use crate::infrastructure::repositories::jwt_repository::StaticJwtRepository;
use crate::infrastructure::repositories::keystore_repository::PostgresKeyStoreRepository;
use crate::infrastructure::repositories::realm_repository::PostgresRealmRepository;
use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;

pub type DefaultJwtService = JwtServiceImpl<
    StaticJwtRepository,
    PostgresRefreshTokenRepository,
    PostgresKeyStoreRepository,
    PostgresRealmRepository,
>;

#[derive(Debug, Clone)]
pub struct JwtServiceImpl<JR, RR, K, R>
where
    JR: JwtRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    pub jwt_repository: JR,
    pub refresh_token_repository: RR,
    pub keystore_repository: K,
    pub realm_repository: R,
}

impl<JR, RR, K, R> JwtServiceImpl<JR, RR, K, R>
where
    JR: JwtRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    pub fn new(
        jwt_repository: JR,
        refresh_token_repository: RR,
        keystore_repository: K,
        realm_repository: R,
    ) -> Self {
        Self {
            jwt_repository,
            refresh_token_repository,
            keystore_repository,
            realm_repository,
        }
    }
}

impl<JR, RR, K, R> JwtService for JwtServiceImpl<JR, RR, K, R>
where
    JR: JwtRepository,
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    async fn generate_token(&self, claims: JwtClaim) -> Result<Jwt, JwtError> {
        self.jwt_repository.generate_jwt_token(&claims).await
    }

    async fn verify_token(&self, token: String) -> Result<JwtClaim, JwtError> {
        self.jwt_repository.verify_token(token).await
    }

    async fn verify_refresh_token(&self, token: String) -> Result<JwtClaim, JwtError> {
        let claims = self.jwt_repository.verify_token(token).await?;

        let refresh_token = self.refresh_token_repository.get_by_jti(claims.jti).await?;

        if refresh_token.revoked {
            return Err(JwtError::ExpiredToken);
        }

        if let Some(expires_at) = refresh_token.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(JwtError::ExpiredToken);
            }
        }

        Ok(claims)
    }

    async fn generate_refresh_token(&self, refresh_claims: JwtClaim) -> Result<Jwt, JwtError> {
        self.jwt_repository
            .generate_jwt_token(&refresh_claims)
            .await
    }

    async fn retrieve_realm_rsa_keys(&self, realm: &Realm) -> Result<JwtKeyPair, JwtError> {
        self.keystore_repository.get_or_generate_key(realm.id).await
    }
}
