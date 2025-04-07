use crate::domain::credential::entities::model::CredentialData;
use crate::domain::crypto::entities::hash_result::HashResult;

pub trait CryptoService: Clone + Send + Sync + 'static {
    fn hash_password(
        &self,
        password: &str,
    ) -> impl Future<Output = Result<HashResult, anyhow::Error>> + Send;
    fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        credential_data: &CredentialData,
        salt: &str,
    ) -> impl Future<Output = Result<bool, anyhow::Error>> + Send;
}
