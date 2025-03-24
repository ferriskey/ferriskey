use uuid::Uuid;

use super::entities::{error::CredentialError, model::Credential};

pub trait CredentialService: Clone + Send + Sync + 'static {
    fn create_password_credential(
        &self,
        user_id: Uuid,
        password: String,
        label: String,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
    fn reset_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> impl Future<Output = Result<(), CredentialError>> + Send;
    fn verify_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> impl Future<Output = Result<bool, CredentialError>> + Send;
}

pub trait CredentialRepository: Clone + Send + Sync + 'static {
    fn create_credential(
        &self,
        user_id: Uuid,
        credential_type: String,
        secret: String,
        credential: String,
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
