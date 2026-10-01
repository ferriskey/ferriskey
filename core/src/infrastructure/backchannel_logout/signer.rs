use std::sync::Arc;

use ferriskey_security::jwt::ports::KeyStoreRepository;
use jsonwebtoken::{Algorithm, Header};

use crate::domain::authentication::backchannel_logout::{
    LOGOUT_TOKEN_TYP, LogoutTokenClaims, LogoutTokenSigner,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// Signs logout tokens with the realm key that signs every other token, so a
/// client verifies them against the same JWKS.
#[derive(Debug, Clone)]
pub struct KeystoreLogoutTokenSigner<K: KeyStoreRepository> {
    keystore: Arc<K>,
}

impl<K: KeyStoreRepository> KeystoreLogoutTokenSigner<K> {
    pub fn new(keystore: Arc<K>) -> Self {
        Self { keystore }
    }
}

impl<K: KeyStoreRepository> LogoutTokenSigner for KeystoreLogoutTokenSigner<K> {
    async fn sign(
        &self,
        realm_id: RealmId,
        claims: LogoutTokenClaims,
    ) -> Result<String, CoreError> {
        let key_pair = self
            .keystore
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(key_pair.id.to_string());
        header.typ = Some(LOGOUT_TOKEN_TYP.to_string());

        jsonwebtoken::encode(&header, &claims, &key_pair.encoding_key)
            .map_err(|err| CoreError::TokenGenerationError(err.to_string()))
    }
}
