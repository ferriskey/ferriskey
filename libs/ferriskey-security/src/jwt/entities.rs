use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs8::{DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    traits::PublicKeyParts,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::SecurityError;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClaimsTyp {
    Refresh,
    Bearer,
    Temporary,
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
    pub scope: Option<String>,
    pub exp: Option<i64>,

    // Identity claims
    pub preferred_username: Option<String>,
    pub email: Option<String>,

    pub client_id: Option<String>,
}

pub trait TokenClaims: Serialize {
    fn get_exp(&self) -> i64;
    fn get_sub(&self) -> Uuid;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: Uuid,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: Option<i64>,

    // Identity claims
    pub preferred_username: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

impl TokenClaims for IdTokenClaims {
    fn get_exp(&self) -> i64 {
        self.exp
    }

    fn get_sub(&self) -> Uuid {
        self.sub
    }
}

impl JwtClaim {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sub: Uuid,
        preferred_username: String,
        iss: String,
        aud: Vec<String>,
        typ: ClaimsTyp,
        azp: String,
        email: Option<String>,
        scope: Option<String>,
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
            scope,
            client_id: None,
        }
    }

    pub fn new_refresh_token(
        sub: Uuid,
        iss: String,
        aud: Vec<String>,
        azp: String,
        scope: Option<String>,
    ) -> Self {
        Self {
            sub,
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
            iss,
            aud,
            typ: ClaimsTyp::Refresh,
            azp,
            scope,
            preferred_username: None,
            email: None,
            exp: Some(chrono::Utc::now().timestamp() + 86400), // 24 hours
            client_id: None,
        }
    }

    pub fn new_temporary_token(claims: JwtClaim) -> Self {
        Self {
            sub: claims.sub,
            iat: claims.iat,
            jti: claims.jti,
            iss: claims.iss,
            aud: claims.aud,
            typ: ClaimsTyp::Temporary,
            azp: claims.azp,
            scope: claims.scope,
            preferred_username: claims.preferred_username,
            email: claims.email,
            exp: Some(chrono::Utc::now().timestamp() + 300), // 5 minutes
            client_id: claims.client_id,
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
    pub x5c: Vec<String>,
    pub x5t: String,
    pub n: String,
    pub e: String,
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

pub struct AccessToken {
    pub id: Uuid,
    pub token_hash: String,
    pub jti: Option<Uuid>,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub claims: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl AccessToken {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: Uuid,
        revoked: bool,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            token_hash,
            jti,
            user_id,
            realm_id,
            revoked,
            expires_at,
            claims,
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
    ) -> Result<Self, SecurityError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        Ok(Self {
            id,
            realm_id,
            encoding_key,
            decoding_key,
            public_key: public_pem.to_string(),
        })
    }

    pub fn generate() -> Result<(String, String), SecurityError> {
        let mut rng = rand::thread_rng();
        let bits = 2048;

        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?
            .to_string();

        let public_pem = private_key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        Ok((private_pem, public_pem))
    }

    pub fn to_jwk_key(&self) -> Result<JwkKey, SecurityError> {
        let public_key = RsaPublicKey::from_public_key_pem(&self.public_key)
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        let n = BASE64_URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = BASE64_URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());
        let x5c_value = BASE64_URL_SAFE_NO_PAD.encode(self.public_key.as_bytes());
        let x5t = BASE64_URL_SAFE_NO_PAD.encode(Sha1::digest(x5c_value.as_bytes()));

        Ok(JwkKey {
            kid: self.id.to_string(),
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            x5c: vec![x5c_value],
            x5t,
            n,
            e,
        })
    }

    pub fn to_jwt_key(&self) -> Result<JwkKey, SecurityError> {
        self.to_jwk_key()
    }
}

#[cfg(test)]
mod tests {
    use super::{ClaimsTyp, JwtClaim};
    use uuid::Uuid;

    #[test]
    fn refresh_claims_keep_scope() {
        let scope = Some("openid profile email".to_string());
        let claims = JwtClaim::new_refresh_token(
            Uuid::new_v4(),
            "https://issuer".to_string(),
            vec!["test-realm".to_string()],
            "client-id".to_string(),
            scope.clone(),
        );

        assert_eq!(claims.typ, ClaimsTyp::Refresh);
        assert_eq!(claims.scope, scope);
    }
}
