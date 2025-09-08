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
