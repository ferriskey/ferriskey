use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::trident::entities::{
    WebAuthnAttestationObject, WebAuthnAuthenticatorTransport, WebAuthnCredentialId,
    WebAuthnPublicKey,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Credential {
    pub id: Uuid,
    pub salt: Option<String>,
    pub credential_type: String,
    pub user_id: Uuid,
    pub user_label: Option<String>,
    pub secret_data: String,
    pub credential_data: CredentialData,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webauthn_credential_id: Option<WebAuthnCredentialId>,
    pub webauthn_public_key: Option<WebAuthnPublicKey>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CredentialOverview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_type: String,
    pub user_label: Option<String>,
    pub credential_data: CredentialData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Credential> for CredentialOverview {
    fn from(credential: Credential) -> Self {
        Self {
            id: credential.id,
            user_id: credential.user_id,
            credential_type: credential.credential_type,
            user_label: credential.user_label,
            credential_data: credential.credential_data,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        }
    }
}

impl Credential {
    pub fn new(config: CredentialConfig) -> Self {
        Self {
            id: config.id,
            salt: config.salt,
            credential_type: config.credential_type,
            user_id: config.user_id,
            user_label: config.user_label,
            secret_data: config.secret_data,
            credential_data: config.credential_data,
            temporary: config.temporary,
            created_at: config.created_at,
            updated_at: config.updated_at,
            webauthn_credential_id: config.webauthn_credential_id,
            webauthn_public_key: config.webauthn_public_key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
#[serde(untagged)]
pub enum CredentialData {
    Hash {
        hash_iterations: u32,
        algorithm: String,
    },
    WebAuthn {
        attestation_object: WebAuthnAttestationObject,
        transports: Vec<WebAuthnAuthenticatorTransport>,
    },
}

impl CredentialData {
    pub fn new_hash(hash_iterations: u32, algorithm: String) -> Self {
        Self::Hash {
            hash_iterations,
            algorithm,
        }
    }

    pub fn new_webauthn(
        attestation_object: WebAuthnAttestationObject,
        transports: Vec<WebAuthnAuthenticatorTransport>,
    ) -> Self {
        Self::WebAuthn {
            attestation_object,
            transports,
        }
    }
}

pub struct CredentialConfig {
    pub id: Uuid,
    pub salt: Option<String>,
    pub credential_type: String,
    pub user_id: Uuid,
    pub user_label: Option<String>,
    pub secret_data: String,
    pub credential_data: CredentialData,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webauthn_credential_id: Option<WebAuthnCredentialId>,
    pub webauthn_public_key: Option<WebAuthnPublicKey>,
}

#[derive(Debug, Clone, Error)]
pub enum CredentialError {
    #[error("Hash password error: {0}")]
    HashPasswordError(String),

    #[error("Verify password error: {0}")]
    VerifyPasswordError(String),

    #[error("Delete password credential error")]
    DeletePasswordCredentialError,

    #[error("Create credential error")]
    CreateCredentialError,

    #[error("Get password credential error")]
    GetPasswordCredentialError,

    #[error("Get user credentials error")]
    GetUserCredentialsError,

    #[error("Delete credential error")]
    DeleteCredentialError,
}

pub struct GetCredentialsInput {
    pub realm_name: String,
    pub user_id: Uuid,
}

pub struct DeleteCredentialInput {
    pub realm_name: String,
    pub credential_id: Uuid,
}
