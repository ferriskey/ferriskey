use uuid::Uuid;

use crate::domain::{
    credential::entities::{Credential, CredentialError},
    crypto::entities::HashResult,
};

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
        credential_type: String,
        secret_data: String,
        label: Option<String>,
        credential_data: serde_json::Value,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;
}

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
