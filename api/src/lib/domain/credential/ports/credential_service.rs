use crate::domain::credential::entities::error::CredentialError;
use crate::domain::credential::entities::model::Credential;
use uuid::Uuid;

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
