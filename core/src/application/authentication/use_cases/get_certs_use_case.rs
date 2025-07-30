use crate::domain::{
    jwt::{
        entities::{JwkKey, JwtError},
        ports::JwtService,
    },
    realm::ports::RealmService,
};

#[derive(Clone)]
pub struct GetCertsUseCase<R, J>
where
    R: RealmService,
    J: JwtService,
{
    pub realm_service: R,
    pub jwt_service: J,
}

impl<R, J> GetCertsUseCase<R, J>
where
    R: RealmService,
    J: JwtService,
{
    pub fn new(realm_service: R, jwt_service: J) -> Self {
        Self {
            realm_service,
            jwt_service,
        }
    }

    pub async fn execute(&self, realm_name: String) -> Result<Vec<JwkKey>, JwtError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)?;

        let jwk_keypair = self
            .jwt_service
            .retrieve_realm_rsa_keys(&realm)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)?;

        let jwk_key = jwk_keypair
            .to_jwk_key()
            .map_err(|e| JwtError::InvalidKey(e.to_string()))?;

        Ok(vec![jwk_key])
    }
}
