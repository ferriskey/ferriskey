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

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;
    mock! {
        pub JwtService {}
        impl Clone for JwtService {
            fn clone(&self) -> Self;
        }
        impl JwtService for JwtService {
            fn generate_token(&self, claims: JwtClaim, realm_id: Uuid) -> impl Future<Output = Result<Jwt, JwtError>> + Send;
            fn verify_token(&self, token: String, realm_id: Uuid) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;
            fn verify_refresh_token(&self, token: String, realm_id: Uuid) -> impl Future<Output = Result<JwtClaim, JwtError>> + Send;
            fn retrieve_realm_rsa_keys(&self, realm: &Realm) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
        }
    }
    pub fn get_mock_jwt_service_with_clone_expectations() -> MockJwtService {
        let mut mock = MockJwtService::new();
        mock.expect_clone()
            .returning(|| get_mock_jwt_service_with_clone_expectations());
        mock
    }

    mock! {
        pub RefreshTokenRepository {}
        impl Clone for RefreshTokenRepository {
            fn clone(&self) -> Self;
        }
        impl RefreshTokenRepository for RefreshTokenRepository {
            fn create(&self, jti: Uuid, user_id: Uuid, expires_at: Option<DateTime<Utc>>) -> impl Future<Output = Result<RefreshToken, JwtError>> + Send;
            fn get_by_jti(&self, jti: Uuid) -> impl Future<Output = Result<RefreshToken, JwtError>> + Send;
            fn delete(&self, jti: Uuid) -> impl Future<Output = Result<(), JwtError>> + Send;
        }
    }
    pub fn get_mock_refresh_token_repository_with_clone_expectations() -> MockRefreshTokenRepository {
        let mut mock = MockRefreshTokenRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_refresh_token_repository_with_clone_expectations());
        mock
    }

    mock! {
        pub KeyStoreRepository {}
        impl Clone for KeyStoreRepository {
            fn clone(&self) -> Self;
        }
        impl KeyStoreRepository for KeyStoreRepository {
            fn get_or_generate_key(&self, realm_id: Uuid) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
        }
    }
    pub fn get_mock_key_store_repository_with_clone_expectations() -> MockKeyStoreRepository {
        let mut mock = MockKeyStoreRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_key_store_repository_with_clone_expectations());
        mock
    }
}