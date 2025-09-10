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
