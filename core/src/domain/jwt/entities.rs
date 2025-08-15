use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClaimsTyp {
    Refresh,
    Bearer,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord)]
pub struct JwtClaim {
    pub sub: Uuid,
    pub iat: i64,
    pub jti: Uuid,
    pub iss: String,
    pub typ: ClaimsTyp,
    pub azp: String,
    pub aud: Vec<String>,

    pub exp: Option<i64>,
    pub preferred_username: Option<String>,

    pub email: Option<String>,

    pub client_id: Option<String>,
}

impl JwtClaim {
    pub fn new(
        sub: Uuid,
        preferred_username: String,
        iss: String,
        aud: Vec<String>,
        typ: ClaimsTyp,
        azp: String,
        email: Option<String>,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        Self {
            sub,
            preferred_username: Some(preferred_username),
            iat: timestamp,
            jti: Uuid::new_v4(),
            exp: Some(timestamp + 60 * 5), // 5 minutes
            iss,
            aud,
            typ,
            azp,
            email,
            client_id: None,
        }
    }

    pub fn new_refresh_token(sub: Uuid, iss: String, aud: Vec<String>, azp: String) -> Self {
        Self {
            sub,
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
            iss,
            aud,
            typ: ClaimsTyp::Refresh,
            azp,
            preferred_username: None,
            email: None,
            exp: Some(chrono::Utc::now().timestamp() + 86400), // 24 hours
            client_id: None,
        }
    }

    pub fn is_service_account(&self) -> bool {
        self.client_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord)]
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

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token generation error: {0}")]
    GenerationError(String),

    #[error("Token validation error: {0}")]
    ValidationError(String),

    #[error("Token parsing error: {0}")]
    ParsingError(String),

    #[error("Token expiration error: {0}")]
    ExpirationError(String),

    #[error("Realm key not found")]
    RealmKeyNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Expired token")]
    ExpiredToken,

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

pub struct RefreshToken {
    pub id: Uuid,
    pub jti: Uuid,
    pub user_id: Uuid,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(
        id: Uuid,
        jti: Uuid,
        user_id: Uuid,
        revoked: bool,
        expires_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            jti,
            user_id,
            revoked,
            expires_at,
            created_at,
        }
    }
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
        let mut rng = rand::thread_rng();
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
