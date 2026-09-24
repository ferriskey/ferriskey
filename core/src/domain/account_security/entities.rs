use chrono::{DateTime, Utc};
use ferriskey_domain::credential::entities::CredentialType;
use uuid::Uuid;
use webauthn_rs::prelude::{CreationChallengeResponse, RegisterPublicKeyCredential};

use crate::domain::trident::ports::WebAuthnRpInfo;

pub enum ElevationProof {
    Password(String),
    Otp(String),
}

pub struct RequestElevationInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub proof: ElevationProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestElevationOutput {
    pub elevation_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

pub struct ChangeOwnPasswordInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartOwnOtpEnrollmentInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub issuer: String,
}

pub struct StartOwnOtpEnrollmentOutput {
    pub secret: String,
    pub otpauth_uri: String,
}

pub struct ConfirmOwnOtpEnrollmentInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub code: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableOwnOtpInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListOwnCredentialsInput {
    pub realm_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnCredential {
    pub id: Uuid,
    pub credential_type: CredentialType,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteOwnPasskeyInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub credential_id: Uuid,
}

pub struct StartOwnPasskeyRegistrationInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub rp_info: WebAuthnRpInfo,
}

pub struct StartOwnPasskeyRegistrationOutput(pub CreationChallengeResponse);

pub struct ConfirmOwnPasskeyRegistrationInput {
    pub realm_name: String,
    pub session_id: Uuid,
    pub elevation_id: Uuid,
    pub rp_info: WebAuthnRpInfo,
    pub credential: RegisterPublicKeyCredential,
}
