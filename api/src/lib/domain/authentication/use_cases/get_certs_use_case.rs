use crate::domain::{
    jwt::{
        entities::{jwt::JwkKey, jwt_error::JwtError},
        ports::jwt_service::JwtService,
        services::jwt_service::DefaultJwtService,
    },
    realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
};

#[derive(Clone)]
pub struct GetCertsUseCase {
    pub realm_service: DefaultRealmService,
    pub jwt_service: DefaultJwtService,
}

impl GetCertsUseCase {
    pub fn new(realm_service: DefaultRealmService, jwt_service: DefaultJwtService) -> Self {
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
