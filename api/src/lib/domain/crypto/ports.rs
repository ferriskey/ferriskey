use crate::domain::credential::entities::model::CredentialData;

#[derive(Debug, Clone)]
pub struct HashResult {
    pub hash: String,
    pub salt: String,
    pub credential_data: CredentialData,
}

impl HashResult {
    pub fn new(hash: String, salt: String, credential_data: CredentialData) -> Self {
        Self {
            hash,
            salt,
            credential_data,
        }
    }
}

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

pub trait HasherRepository: Clone + Send + Sync + 'static {
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
