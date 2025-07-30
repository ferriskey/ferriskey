use crate::domain::{
    jwt::{
        entities::{Jwt, JwtClaim, JwtError, JwtKeyPair},
        ports::{JwtService, KeyStoreRepository, RefreshTokenRepository},
    },
    realm::{entities::Realm, ports::RealmRepository},
};
use jsonwebtoken::{Algorithm, Header, Validation, decode};
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtServiceImpl<RR, K, R>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    pub refresh_token_repository: RR,
    pub keystore_repository: K,
    pub realm_repository: R,
}

impl<RR, K, R> JwtServiceImpl<RR, K, R>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    pub fn new(refresh_token_repository: RR, keystore_repository: K, realm_repository: R) -> Self {
        Self {
            refresh_token_repository,
            keystore_repository,
            realm_repository,
        }
    }
}

impl<RR, K, R> JwtService for JwtServiceImpl<RR, K, R>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
{
    async fn generate_token(&self, claims: JwtClaim, realm_id: Uuid) -> Result<Jwt, JwtError> {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await?;

        let header = Header::new(Algorithm::RS256);
        let token =
            jsonwebtoken::encode(&header, &claims, &jwt_key_pair.encoding_key).map_err(|e| {
                error!("JWT generation error: {}", e);

                JwtError::GenerationError(e.to_string())
            })?;

        let exp = claims.exp.unwrap_or(0);

        Ok(Jwt {
            token,
            expires_at: exp,
        })
    }

    async fn verify_token(&self, token: String, realm_id: Uuid) -> Result<JwtClaim, JwtError> {
        let mut validation = Validation::new(Algorithm::RS256);

        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await?;

        validation.validate_aud = false;
        let token_data = decode::<JwtClaim>(&token, &jwt_key_pair.decoding_key, &validation)
            .map_err(|e| JwtError::ValidationError(e.to_string()))?;

        let current_time = chrono::Utc::now().timestamp();

        if let Some(exp) = token_data.claims.exp {
            if exp < current_time {
                return Err(JwtError::ExpiredToken);
            }
        }

        Ok(token_data.claims)
    }

    async fn verify_refresh_token(
        &self,
        token: String,
        realm_id: Uuid,
    ) -> Result<JwtClaim, JwtError> {
        let claims = self.verify_token(token, realm_id).await?;

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

    async fn retrieve_realm_rsa_keys(&self, realm: &Realm) -> Result<JwtKeyPair, JwtError> {
        self.keystore_repository.get_or_generate_key(realm.id).await
    }
}
