use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;
use webauthn_rs::prelude::{Credential as WebAuthnCredential, CredentialID, Passkey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: Uuid,
    pub salt: Option<String>,
    pub credential_type: CredentialType,
    pub user_id: Uuid,
    pub user_label: Option<String>,
    pub secret_data: String,
    pub credential_data: CredentialData,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webauthn_credential_id: Option<CredentialID>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum CredentialType {
    Password,
    Otp,
    RecoveryCode,
    WebAuthnPublicKeyCredential,
}

impl Display for CredentialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CredentialType::Password => "password",
            CredentialType::Otp => "otp",
            CredentialType::RecoveryCode => "recovery-code",
            CredentialType::WebAuthnPublicKeyCredential => "webauthn-public-key-credential",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for CredentialType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "password" => CredentialType::Password,
            "otp" => CredentialType::Otp,
            "recovery-code" => CredentialType::RecoveryCode,
            "webauthn-public-key-credential" => CredentialType::WebAuthnPublicKeyCredential,
            _ => CredentialType::Password, // default to Password if unknown
        }
    }
}

impl CredentialType {
    pub fn as_str(&self) -> &str {
        match self {
            CredentialType::Password => "password",
            CredentialType::Otp => "otp",
            CredentialType::RecoveryCode => "recovery-code",
            CredentialType::WebAuthnPublicKeyCredential => "webauthn-public-key-credential",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, ToSchema)]
pub struct CredentialOverview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_type: String,
    pub user_label: Option<String>,
    pub credential_data: CredentialDataOverview,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Credential> for CredentialOverview {
    fn from(credential: Credential) -> Self {
        Self {
            id: credential.id,
            user_id: credential.user_id,
            credential_type: credential.credential_type.to_string(),
            user_label: credential.user_label,
            credential_data: credential.credential_data.into(),
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
            credential_type: config.credential_type.into(),
            user_id: config.user_id,
            user_label: config.user_label,
            secret_data: config.secret_data,
            credential_data: config.credential_data,
            temporary: config.temporary,
            created_at: config.created_at,
            updated_at: config.updated_at,
            webauthn_credential_id: config.webauthn_credential_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// for back-compat, maybe we should switch this to tagged in the future ?
#[serde(untagged)]
pub enum CredentialData {
    Hash {
        hash_iterations: u32,
        algorithm: String,
    },
    WebAuthn {
        credential: Box<WebAuthnCredential>,
    },
    Federated {
        provider_id: String,
        provider_type: String,
    },
}

impl CredentialData {
    pub fn new_hash(hash_iterations: u32, algorithm: String) -> Self {
        Self::Hash {
            hash_iterations,
            algorithm,
        }
    }

    pub fn new_webauthn(passkey: Passkey) -> Self {
        Self::WebAuthn {
            credential: Box::new(passkey.into()),
        }
    }

    pub fn new_federated(provider_id: String, provider_type: String) -> Self {
        Self::Federated {
            provider_id,
            provider_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, ToSchema)]
pub enum CredentialDataOverview {
    Hash {
        hash_iterations: u32,
        algorithm: String,
    },
    // It's not inherently risky to reveal some WA credential data but it's a bit unusual...
    // I'll leave it empty for now
    WebAuthn,
    Federated {
        provider_id: String,
        provider_type: String,
    },
}

impl From<CredentialData> for CredentialDataOverview {
    fn from(data: CredentialData) -> Self {
        match data {
            CredentialData::Hash {
                hash_iterations,
                algorithm,
            } => CredentialDataOverview::Hash {
                hash_iterations,
                algorithm,
            },
            CredentialData::WebAuthn { .. } => CredentialDataOverview::WebAuthn,
            CredentialData::Federated {
                provider_id,
                provider_type,
            } => CredentialDataOverview::Federated {
                provider_id,
                provider_type,
            },
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
    pub webauthn_credential_id: Option<CredentialID>,
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

    #[error("Update credential error")]
    UpdateCredentialError,

    #[error("Unexpected credential data variant")]
    UnexpectedCredentialData,
}

pub struct GetCredentialsInput {
    pub realm_name: String,
    pub user_id: Uuid,
}

pub struct DeleteCredentialInput {
    pub realm_name: String,
    pub credential_id: Uuid,
}
