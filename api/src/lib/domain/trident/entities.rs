use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TotpCredentialData {
    pub algorithm: String,
    pub digits: u32,
    pub period: u64,
    pub issuer: String,
    pub account_name: String,
}

#[derive(Debug, Clone, Error)]
pub enum TotpError {
    #[error("Invalid TOTP secret format")]
    InvalidSecretFormat,
    #[error("TOTP generation failed: {0}")]
    GenerationFailed(String),
    #[error("TOTP verification failed: {0}")]
    VerificationFailed(String),
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

    pub fn to_bytes(&self) -> Result<[u8; 20], TotpError> {
        let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &self.base32)
            .ok_or(TotpError::InvalidSecretFormat)?;

        if decoded.len() != 20 {
            return Err(TotpError::InvalidSecretFormat);
        }

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded);
        Ok(bytes)
    }
}
