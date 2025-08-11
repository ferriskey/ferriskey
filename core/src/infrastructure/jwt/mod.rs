use crate::domain::jwt::entities::{JwtError, JwtKeyPair};
use crate::domain::jwt::ports::KeyStoreRepository;
use crate::infrastructure::repositories::keystore_repository::PostgresKeyStoreRepository;
use uuid::Uuid;

#[derive(Clone)]
pub enum KeyStoreRepoAny {
    Postgres(PostgresKeyStoreRepository),
}

impl KeyStoreRepository for KeyStoreRepoAny {
    async fn get_or_generate_key(&self, realm_id: Uuid) -> Result<JwtKeyPair, JwtError> {
        match self {
            KeyStoreRepoAny::Postgres(repo) => repo.get_or_generate_key(realm_id).await,
        }
    }
}
