use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    credential::entities::{
        Credential, CredentialError, CredentialOverview, DeleteCredentialInput, GetCredentialsInput,
    },
    crypto::entities::HashResult,
};

pub trait CredentialService: Clone + Send + Sync + 'static {
    fn get_credentials(
        &self,
        identity: Identity,
        input: GetCredentialsInput,
    ) -> impl Future<Output = Result<Vec<CredentialOverview>, CoreError>> + Send;
    fn delete_credential(
        &self,
        identity: Identity,
        input: DeleteCredentialInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait CredentialRepository: Clone + Send + Sync + 'static {
    fn create_credential(
        &self,
        user_id: Uuid,
        credential_type: String,
        hash_result: HashResult,
        label: String,
        temporary: bool,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;

    fn get_password_credential(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;

    fn delete_password_credential(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CredentialError>> + Send;

    fn get_credentials_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Credential>, CredentialError>> + Send;
    fn delete_by_id(
        &self,
        credential_id: Uuid,
    ) -> impl Future<Output = Result<(), CredentialError>> + Send;
    fn create_custom_credential(
        &self,
        user_id: Uuid,
        credential_type: String, // "TOTP", "WEBAUTHN", etc.
        secret_data: String,     // base32 pour TOTP
        label: Option<String>,
        credential_data: serde_json::Value,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
}

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;

    mock! {
        pub CredentialService {}
        impl Clone for CredentialService { fn clone(&self) -> Self; }
        impl CredentialService for CredentialService {
            fn get_credentials(&self, identity: Identity, input: GetCredentialsInput) -> impl Future<Output = Result<Vec<CredentialOverview>, CoreError>> + Send;
            fn delete_credential(&self, identity: Identity, input: DeleteCredentialInput) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }
    pub fn get_mock_credential_service_with_clone_expectations() -> MockCredentialService {
        let mut mock = MockCredentialService::new();
        mock.expect_clone()
            .returning(|| get_mock_credential_service_with_clone_expectations());
        mock
    }

    mock! {
        pub CredentialRepository {}
        impl Clone for CredentialRepository { fn clone(&self) -> Self; }
        impl CredentialRepository for CredentialRepository {
            fn create_credential(&self, user_id: Uuid, credential_type: String, hash_result: HashResult, label: String, temporary: bool) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
            fn get_password_credential(&self, user_id: Uuid) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
            fn delete_password_credential(&self, user_id: Uuid) -> impl Future<Output = Result<(), CredentialError>> + Send;
            fn get_credentials_by_user_id(&self, user_id: Uuid) -> impl Future<Output = Result<Vec<Credential>, CredentialError>> + Send;
            fn delete_by_id(&self, credential_id: Uuid) -> impl Future<Output = Result<(), CredentialError>> + Send;
            fn create_custom_credential(&self, user_id: Uuid, credential_type: String, secret_data: String, label: Option<String>, credential_data: serde_json::Value) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
        }
    }
    pub fn get_mock_credential_repository_with_clone_expectations() -> MockCredentialRepository {
        let mut mock = MockCredentialRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_credential_repository_with_clone_expectations());
        mock
    }
}
