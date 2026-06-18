// aes-gcm 0.10 re-exports the older generic-array 0.14 types; the deprecation
// warnings below are cosmetic and safe to silence until aes-gcm publishes a
// 0.11 release based on generic-array 1.x.
#[allow(deprecated)]
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, AeadCore, KeySizeUser, OsRng, generic_array::GenericArray},
};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::SecurityError;

/// Blob layout stored in the database (base64-encoded):
///
/// ```text
/// | key_id (1 byte) | nonce (12 bytes) | ciphertext + GCM tag |
/// ```
///
/// `key_id` is a single byte identifying which key was used. Value `0` is
/// reserved as a sentinel meaning "plaintext — not yet encrypted". Values
/// `1..=255` reference versioned master keys.
///
/// Backfill: existing rows with `secret_key_id IS NULL` are read as plaintext.
/// A future migration job sets `secret_key_id = "v1"` after re-encrypting.
pub const SENTINEL_PLAINTEXT: u8 = 0;
pub const KEY_ID_V1: u8 = 1;

/// AES-256-GCM authenticated field cipher. Thread-safe and cheap to clone.
#[derive(Clone)]
pub struct FieldCipher {
    key_id: u8,
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for FieldCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldCipher")
            .field("key_id", &self.key_id)
            .field("cipher", &"<AES-256-GCM>")
            .finish()
    }
}

impl FieldCipher {
    /// Build from a 32-byte master key and a key identifier byte.
    pub fn new(key_bytes: &[u8; 32], key_id: u8) -> Self {
        #[allow(deprecated)]
        let key: &GenericArray<u8, <Aes256Gcm as KeySizeUser>::KeySize> =
            GenericArray::from_slice(key_bytes);
        Self {
            key_id,
            cipher: Aes256Gcm::new(key),
        }
    }

    /// Encrypt `plaintext` and return a base64-encoded blob.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, SecurityError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| SecurityError::GenerationError(e.to_string()))?;

        let mut blob = Vec::with_capacity(1 + 12 + ciphertext.len());
        blob.push(self.key_id);
        #[allow(deprecated)]
        blob.extend_from_slice(nonce.as_ref());
        blob.extend_from_slice(&ciphertext);

        Ok(STANDARD.encode(&blob))
    }

    /// Decrypt a base64-encoded blob produced by [`FieldCipher::encrypt`].
    pub fn decrypt(&self, blob: &str) -> Result<String, SecurityError> {
        let raw = STANDARD
            .decode(blob)
            .map_err(|e| SecurityError::ParsingError(e.to_string()))?;

        if raw.len() < 1 + 12 + 16 {
            return Err(SecurityError::ParsingError(
                "encrypted blob too short".to_string(),
            ));
        }

        let _stored_key_id = raw[0];
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&raw[1..13]);
        let ciphertext = &raw[13..];

        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SecurityError::ValidationError("decryption failed".to_string()))?;

        String::from_utf8(plaintext_bytes).map_err(|e| SecurityError::ParsingError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        *b"an example very very secret key!"
    }

    fn cipher() -> FieldCipher {
        FieldCipher::new(&test_key(), KEY_ID_V1)
    }

    #[test]
    fn field_cipher_round_trip() {
        let c = cipher();
        let plaintext = "super-secret-client-password-123";
        let blob = c.encrypt(plaintext).expect("encrypt");
        let recovered = c.decrypt(&blob).expect("decrypt");
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn field_cipher_tamper_detection() {
        let c = cipher();
        let blob = c.encrypt("sensitive-value").expect("encrypt");

        let mut raw = base64::engine::general_purpose::STANDARD
            .decode(&blob)
            .unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0xff;
        let tampered = STANDARD.encode(&raw);

        let result = c.decrypt(&tampered);
        assert!(
            result.is_err(),
            "tampered ciphertext should not decrypt successfully"
        );
    }

    #[test]
    fn field_cipher_wrong_key() {
        let c = cipher();
        let blob = c.encrypt("another-secret").expect("encrypt");

        let wrong_key = *b"totally different key totally!!!";
        let wrong_cipher = FieldCipher::new(&wrong_key, KEY_ID_V1);

        let result = wrong_cipher.decrypt(&blob);
        assert!(result.is_err(), "wrong key should not decrypt successfully");
    }

    #[test]
    fn field_cipher_empty_blob_rejected() {
        let c = cipher();
        let result = c.decrypt("");
        assert!(result.is_err());
    }
}
