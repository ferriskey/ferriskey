use crate::domain::credential::entities::error::CredentialError;
use crate::domain::credential::entities::model::Credential;
use crate::domain::crypto::entities::hash_result::HashResult;
use uuid::Uuid;

pub trait CredentialRepository: Clone + Send + Sync + 'static {
    fn create_credential(
        &self,
        user_id: Uuid,
        credential_type: String,
        hash_result: HashResult,
        label: String,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;

    fn get_password_credential(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;

    fn delete_password_credential(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CredentialError>> + Send;
}
