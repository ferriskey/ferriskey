use uuid::Uuid;
use webauthn_rs::prelude::{CredentialID, Passkey};

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    credential::entities::{
        Credential, CredentialError, CredentialOverview, DeleteCredentialInput, GetCredentialsInput,
    },
    crypto::entities::HashResult,
};

pub trait CredentialService: Send + Sync {
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

#[cfg_attr(test, mockall::automock)]
pub trait CredentialRepository: Send + Sync {
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

    fn create_recovery_code_credentials(
        &self,
        user_id: Uuid,
        hash: Vec<HashResult>,
    ) -> impl Future<Output = Result<(), CredentialError>> + Send;

    fn create_webauthn_credential(
        &self,
        user_id: Uuid,
        webauthn_credential: Passkey,
    ) -> impl Future<Output = Result<Credential, CredentialError>> + Send;

    fn get_webauthn_public_key_credentials(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Credential>, CredentialError>> + Send;

    fn get_webauthn_credential_by_credential_id(
        &self,
        webauthn_credential_id: CredentialID,
    ) -> impl Future<Output = Result<Option<Credential>, CredentialError>> + Send;
}
