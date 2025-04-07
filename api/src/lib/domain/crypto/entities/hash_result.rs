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