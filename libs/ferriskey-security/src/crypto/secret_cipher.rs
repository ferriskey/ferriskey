use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use aws_lc_rs::rand::fill;
use base64::{
    Engine as _, engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD,
};
use thiserror::Error;

const ENVELOPE_PREFIX: &str = "fk1:";
const KEY_LEN: usize = 32;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecretCipherError {
    #[error("invalid secrets encryption key ring: {0}")]
    InvalidKeyRing(String),

    #[error("the secret is sealed with key {0}, which is not in the key ring")]
    UnknownKey(String),

    #[error("the secret is sealed but no encryption key is configured")]
    KeyRingMissing,

    #[error("the sealed secret is malformed")]
    Malformed,

    #[error("the secret could not be sealed")]
    SealFailed,

    #[error("the secret could not be opened: wrong key, place or tampered value")]
    OpenFailed,
}

struct Keyring {
    active: String,
    keys: HashMap<String, LessSafeKey>,
}

#[derive(Clone)]
pub struct SecretCipher {
    keyring: Option<Arc<Keyring>>,
}

impl fmt::Debug for SecretCipher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.keyring {
            None => f.write_str("SecretCipher(disabled)"),
            Some(keyring) => {
                let mut ids: Vec<&String> = keyring.keys.keys().collect();
                ids.sort();
                f.debug_struct("SecretCipher")
                    .field("active", &keyring.active)
                    .field("keys", &ids)
                    .finish()
            }
        }
    }
}

fn aad(purpose: &str, context: &str) -> Vec<u8> {
    format!("fk1|{purpose}|{context}").into_bytes()
}

fn valid_key_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

impl SecretCipher {
    pub fn disabled() -> Self {
        Self { keyring: None }
    }

    pub fn from_config(spec: &str) -> Result<Self, SecretCipherError> {
        let spec = spec.trim();
        if spec.is_empty() {
            return Ok(Self::disabled());
        }

        let mut keys = HashMap::new();
        let mut active = None;

        for entry in spec.split(',') {
            let (id, encoded) = entry.trim().split_once(':').ok_or_else(|| {
                SecretCipherError::InvalidKeyRing("each key must be written id:base64".to_string())
            })?;

            if !valid_key_id(id) {
                return Err(SecretCipherError::InvalidKeyRing(format!(
                    "key id {id:?} may only hold letters, digits, '-' and '_'"
                )));
            }

            let bytes = STANDARD.decode(encoded.trim()).map_err(|_| {
                SecretCipherError::InvalidKeyRing(format!("key {id} is not valid base64"))
            })?;
            if bytes.len() != KEY_LEN {
                return Err(SecretCipherError::InvalidKeyRing(format!(
                    "key {id} must hold {KEY_LEN} bytes"
                )));
            }

            let unbound = UnboundKey::new(&AES_256_GCM, &bytes).map_err(|_| {
                SecretCipherError::InvalidKeyRing(format!("key {id} was refused by the cipher"))
            })?;

            if keys
                .insert(id.to_string(), LessSafeKey::new(unbound))
                .is_some()
            {
                return Err(SecretCipherError::InvalidKeyRing(format!(
                    "key id {id} appears twice"
                )));
            }
            active.get_or_insert_with(|| id.to_string());
        }

        Ok(Self {
            keyring: active.map(|active| Arc::new(Keyring { active, keys })),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.keyring.is_some()
    }

    pub fn active_key_id(&self) -> Option<&str> {
        self.keyring.as_ref().map(|keyring| keyring.active.as_str())
    }

    pub fn is_sealed(value: &str) -> bool {
        value.starts_with(ENVELOPE_PREFIX)
    }

    pub fn needs_sealing(&self, value: &str) -> bool {
        let Some(keyring) = &self.keyring else {
            return false;
        };

        match value.strip_prefix(ENVELOPE_PREFIX) {
            None => true,
            Some(rest) => rest
                .split_once(':')
                .is_none_or(|(id, _)| id != keyring.active),
        }
    }

    pub fn seal(
        &self,
        purpose: &str,
        context: &str,
        plaintext: &str,
    ) -> Result<String, SecretCipherError> {
        let Some(keyring) = &self.keyring else {
            return Ok(plaintext.to_string());
        };

        let key = keyring
            .keys
            .get(&keyring.active)
            .ok_or(SecretCipherError::SealFailed)?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        fill(&mut nonce_bytes).map_err(|_| SecretCipherError::SealFailed)?;

        let mut in_out = plaintext.as_bytes().to_vec();
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::from(aad(purpose, context)),
            &mut in_out,
        )
        .map_err(|_| SecretCipherError::SealFailed)?;

        let mut payload = Vec::with_capacity(NONCE_LEN + in_out.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&in_out);

        Ok(format!(
            "{ENVELOPE_PREFIX}{}:{}",
            keyring.active,
            URL_SAFE_NO_PAD.encode(payload)
        ))
    }

    pub fn open(
        &self,
        purpose: &str,
        context: &str,
        stored: &str,
    ) -> Result<String, SecretCipherError> {
        let Some(rest) = stored.strip_prefix(ENVELOPE_PREFIX) else {
            return Ok(stored.to_string());
        };

        let keyring = self
            .keyring
            .as_ref()
            .ok_or(SecretCipherError::KeyRingMissing)?;

        let (id, encoded) = rest.split_once(':').ok_or(SecretCipherError::Malformed)?;
        let key = keyring
            .keys
            .get(id)
            .ok_or_else(|| SecretCipherError::UnknownKey(id.to_string()))?;

        let payload = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| SecretCipherError::Malformed)?;
        if payload.len() < NONCE_LEN {
            return Err(SecretCipherError::Malformed);
        }

        let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| SecretCipherError::Malformed)?;

        let mut in_out = ciphertext.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::from(aad(purpose, context)), &mut in_out)
            .map_err(|_| SecretCipherError::OpenFailed)?;

        String::from_utf8(plaintext.to_vec()).map_err(|_| SecretCipherError::OpenFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_A: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
    const KEY_B: &str = "kid-b:ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=";

    fn cipher(spec: &str) -> SecretCipher {
        SecretCipher::from_config(spec).expect("a valid key ring")
    }

    #[test]
    fn a_sealed_value_opens_to_what_was_sealed() {
        let cipher = cipher(KEY_A);

        let sealed = cipher
            .seal("jwt_keys.private_key", "row-1", "pem-body")
            .unwrap();

        assert!(SecretCipher::is_sealed(&sealed));
        assert!(!sealed.contains("pem-body"));
        assert_eq!(
            cipher
                .open("jwt_keys.private_key", "row-1", &sealed)
                .unwrap(),
            "pem-body"
        );
    }

    #[test]
    fn two_seals_of_one_value_differ() {
        let cipher = cipher(KEY_A);

        let first = cipher.seal("p", "c", "same").unwrap();
        let second = cipher.seal("p", "c", "same").unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn a_value_sealed_for_one_place_does_not_open_in_another() {
        let cipher = cipher(KEY_A);
        let sealed = cipher.seal("credentials.otp", "row-1", "JBSWY3DP").unwrap();

        assert!(cipher.open("credentials.otp", "row-2", &sealed).is_err());
        assert!(
            cipher
                .open("smtp_configs.password", "row-1", &sealed)
                .is_err()
        );
    }

    #[test]
    fn a_tampered_value_is_refused() {
        let cipher = cipher(KEY_A);
        let sealed = cipher.seal("p", "c", "secret").unwrap();
        let mut bytes = sealed.into_bytes();
        let last = bytes.len() - 1;
        bytes[last] = if bytes[last] == b'A' { b'B' } else { b'A' };
        let tampered = String::from_utf8(bytes).unwrap();

        assert!(cipher.open("p", "c", &tampered).is_err());
    }

    #[test]
    fn a_value_that_was_never_sealed_passes_through() {
        let cipher = cipher(KEY_A);

        assert_eq!(
            cipher.open("p", "c", "legacy-plaintext").unwrap(),
            "legacy-plaintext"
        );
        assert_eq!(
            SecretCipher::disabled().open("p", "c", "legacy").unwrap(),
            "legacy"
        );
    }

    #[test]
    fn a_disabled_cipher_stores_values_as_they_are() {
        let disabled = SecretCipher::disabled();

        assert!(!disabled.is_enabled());
        assert_eq!(disabled.seal("p", "c", "plain").unwrap(), "plain");
        assert!(!disabled.needs_sealing("plain"));
    }

    #[test]
    fn a_sealed_value_cannot_be_read_without_its_key() {
        let sealed = cipher(KEY_A).seal("p", "c", "secret").unwrap();

        assert!(SecretCipher::disabled().open("p", "c", &sealed).is_err());
        assert!(cipher(KEY_B).open("p", "c", &sealed).is_err());
    }

    #[test]
    fn the_first_key_seals_and_every_key_opens() {
        let old = cipher(KEY_A).seal("p", "c", "secret").unwrap();
        let rotated = cipher(&format!("{KEY_B},{KEY_A}"));

        assert_eq!(rotated.open("p", "c", &old).unwrap(), "secret");
        assert!(
            rotated
                .seal("p", "c", "secret")
                .unwrap()
                .starts_with("fk1:kid-b:")
        );
        assert!(rotated.needs_sealing(&old));
        assert!(!rotated.needs_sealing(&rotated.seal("p", "c", "secret").unwrap()));
        assert!(rotated.needs_sealing("legacy-plaintext"));
    }

    #[test]
    fn a_key_ring_is_validated() {
        for spec in [
            "no-separator",
            "kid:not base64!",
            "kid:AAEC",
            "bad:kid:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=",
            ":AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=",
            "dup:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=,dup:ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=",
        ] {
            assert!(SecretCipher::from_config(spec).is_err(), "{spec}");
        }
    }

    #[test]
    fn the_active_key_is_the_first_of_the_ring() {
        assert_eq!(
            cipher(&format!("{KEY_B},{KEY_A}")).active_key_id(),
            Some("kid-b")
        );
        assert_eq!(SecretCipher::disabled().active_key_id(), None);
    }

    #[test]
    fn an_empty_key_ring_is_a_disabled_cipher() {
        assert!(!SecretCipher::from_config("").unwrap().is_enabled());
        assert!(!SecretCipher::from_config("  ").unwrap().is_enabled());
    }

    #[test]
    fn debug_output_never_shows_key_material() {
        let shown = format!("{:?}", cipher(KEY_A));

        assert!(!shown.contains("AAECAwQF"));
        assert!(shown.contains("kid-a"));
    }
}
