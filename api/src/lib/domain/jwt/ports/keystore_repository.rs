use uuid::Uuid;

use crate::domain::jwt::entities::{jwt::JwtKeyPair, jwt_error::JwtError};

pub trait KeyStoreRepository: Clone + Send + Sync + 'static {
    fn get_or_generate_key(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<JwtKeyPair, JwtError>> + Send;
}
