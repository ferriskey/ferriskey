use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::domain::jwt::{
    entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError},
    ports::jwt_repository::JwtRepository,
};

#[derive(Clone)]
pub struct JwtKeyPair {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

#[derive(Clone)]
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

impl JwtRepository for StaticJwtRepository {
    async fn get_realm_key(&self, _realm: &str) -> Result<String, JwtError> {
        Ok(self.public_key.clone())
    }

    async fn generate_jwt_token(&self, claims: &JwtClaim) -> Result<Jwt, JwtError> {
        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &self.keys.encoding_key)
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;
        Ok(Jwt {
            token,
            expires_at: claims.exp,
        })
    }

    async fn verify_token(&self, token: String) -> Result<JwtClaim, JwtError> {
        let mut validation = Validation::new(Algorithm::RS256);

        validation.validate_aud = false;

        let token_data = decode::<JwtClaim>(&token, &self.keys.decoding_key, &validation)
            .map_err(|e| JwtError::ValidationError(e.to_string()))?;

        let current_time = chrono::Utc::now().timestamp();
        if token_data.claims.exp < current_time {
            return Err(JwtError::ExpiredToken);
        }

        Ok(token_data.claims)
    }
}
