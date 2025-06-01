use jsonwebtoken::{DecodingKey, EncodingKey};
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::jwt_error::JwtError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct Jwt {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Clone)]
pub struct JwtKeyPair {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl JwtKeyPair {
    pub fn from_pem(
        private_pem: &str,
        public_pem: &str,
        realm_id: Uuid,
        id: Uuid,
    ) -> Result<Self, JwtError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .map_err(|e| JwtError::InvalidKey(e.to_string()))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
            .map_err(|e| JwtError::InvalidKey(e.to_string()))?;

        Ok(Self {
            id,
            realm_id,
            encoding_key,
            decoding_key,
        })
    }

    pub fn generate() -> Result<(String, String), JwtError> {
        let mut rng = rand::rngs::OsRng;
        let bits = 2048; // RSA key size in bits
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| JwtError::GenerationError(e.to_string()))?
            .to_string();

        let public_pem = private_key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        Ok((private_pem, public_pem))
    }
}
