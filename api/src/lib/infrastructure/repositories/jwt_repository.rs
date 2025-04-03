use async_trait::async_trait;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode, Algorithm};

use crate::domain::jwt::{entities::{Jwt, JwtError, JwtClaims}, ports::JwtRepository};


pub struct JwtKeyPair {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

pub struct StaticJwtRepository {
    pub keys: JwtKeyPair,
    pub private_key: String,
    pub public_key: String,
}

impl StaticJwtRepository {
    pub fn new(private_pem: &str, public_pem: &str) -> Result<Self, anyhow::Error> {

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())?;

        Ok(Self {
            keys: JwtKeyPair {
                encoding_key,
                decoding_key,
            },
            public_key: public_pem.to_string(),
            private_key: private_pem.to_string(),
        })
    }
}

#[async_trait]
impl JwtRepository for StaticJwtRepository {
    async fn get_realm_key(&self, _realm: &str) -> Result<String, JwtError> {
        Ok(self.public_key.clone())
    }

    async fn generate_jwt_token(&self, claims: &JwtClaims) -> Result<Jwt, JwtError> {
        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &self.keys.encoding_key)
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;
        Ok(Jwt { token, expires_at: claims.exp })
    }
}