use base64::Engine;
use base64::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TotpCredentialData {
    pub algorithm: String,
    pub digits: u32,
    pub period: u64,
    pub issuer: String,
    pub account_name: String,
}

#[derive(Debug, Clone)]
pub struct TotpSecret {
    base32: String,
}

impl TotpSecret {
    pub fn from_base32(base32: &str) -> Self {
        Self {
            base32: base32.to_string(),
        }
    }

    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        let base32 = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes);
        Self { base32 }
    }

    pub fn base32_encoded(&self) -> &str {
        &self.base32
    }

    pub fn to_bytes(&self) -> Result<[u8; 20], CoreError> {
        let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &self.base32)
            .ok_or(CoreError::InvalidTotpSecretFormat)?;

        if decoded.len() != 20 {
            return Err(CoreError::InvalidTotpSecretFormat);
        }

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded);
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MfaRecoveryCode(pub Vec<u8>);

impl MfaRecoveryCode {
    pub fn from_bytes(bytes: &[u8]) -> MfaRecoveryCode {
        MfaRecoveryCode(bytes.to_vec())
    }
}

/// A Webauthn challenge is sent to a user both to create a webauthn credential
/// and to verify an authentication attempt with a webauthn credential
pub struct WebAuthnChallenge(pub Vec<u8>);

impl WebAuthnChallenge {
    /// Generates a random 16 bytes number as a challenge
    /// As specified by the spec, the challenge must be at least 16 bytes long:
    /// https://w3c.github.io/webauthn/#sctn-cryptographic-challenges
    pub fn generate() -> Result<Self, CoreError> {
        let mut bytes = [0u8; 16];
        rand::thread_rng()
            .try_fill_bytes(&mut bytes)
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(WebAuthnChallenge(bytes.to_vec()))
    }

    /// Straight B64 encoding of the challenge
    pub fn encode(&self) -> String {
        BASE64_STANDARD.encode(self.0.clone())
    }
}

impl TryFrom<String> for WebAuthnChallenge {
    type Error = CoreError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let result = BASE64_STANDARD.decode(s).map_err(|_| CoreError::Invalid)?;

        Ok(WebAuthnChallenge(result))
    }
}
