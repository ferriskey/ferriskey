use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use jsonwebtoken::{DecodingKey, EncodingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
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
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct JwkKey {
    pub kid: String,
    pub kty: String,
    pub use_: String,
    pub alg: String,
    pub x5c: String,
    pub n: String,
    pub e: String,
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
            public_key: public_pem.to_string(),
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

    pub fn to_jwk_key(&self) -> Result<JwkKey, JwtError> {
        let public_key = RsaPublicKey::from_public_key_pem(&self.public_key)
            .map_err(|e| JwtError::InvalidKey(e.to_string()))?;

        let n = BASE64_URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = BASE64_URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());
        let x5c = BASE64_URL_SAFE_NO_PAD.encode(self.public_key.as_bytes());

        Ok(JwkKey {
            kid: self.id.to_string(),
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            x5c,
            n,
            e,
        })
    }
}
