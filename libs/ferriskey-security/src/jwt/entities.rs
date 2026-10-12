use std::collections::HashMap;

use aws_lc_rs::{
    encoding::{AsDer, Pkcs8V1Der, PublicKeyX509Der},
    rsa::{KeyPair, KeySize, PublicKey},
    signature::KeyPair as _,
};
use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
};
use chrono::{DateTime, Datelike, Utc};
use ferriskey_domain::realm::{RealmId, scope::RealmOwned};
use jsonwebtoken::{DecodingKey, EncodingKey};
use pem::{EncodeConfig, LineEnding, Pem};
use rcgen::{
    CertificateParams, DistinguishedName, DnType, KeyPair as CertificateKeyPair, KeyUsagePurpose,
    date_time_ymd,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::SecurityError;

/// Default token lifetimes in seconds.
pub const DEFAULT_ACCESS_TOKEN_LIFETIME: i64 = 300; // 5 minutes
pub const DEFAULT_REFRESH_TOKEN_LIFETIME: i64 = 86400; // 24 hours
pub const DEFAULT_ID_TOKEN_LIFETIME: i64 = 300; // 5 minutes
pub const DEFAULT_TEMPORARY_TOKEN_LIFETIME: i64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClaimsTyp {
    Refresh,
    Bearer,
    Temporary,
    #[serde(rename = "ID")]
    Id,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

    // Identity claims — absent when the respective scope is not in the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// OIDC session identifier (`sid`): the `user_sessions` row this token was
    /// issued against. `None` for tokens from flows that establish no SSO
    /// session — `client_credentials`, and any token minted before this claim
    /// existed. Consumers must treat a missing `sid` as "not session-bound"
    /// rather than as an invalid token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<Uuid>,

    /// Dynamic claims injected by protocol mappers.
    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub azp: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub jti: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub nonce: Option<String>,
    pub typ: ClaimsTyp,

    // Identity claims — absent when the respective scope is not active
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,

    /// Dynamic claims injected by protocol mappers.
    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
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
        lifetime_seconds: i64,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        Self {
            sub,
            preferred_username: Some(preferred_username),
            iat: timestamp,
            jti: Uuid::new_v4(),
            exp: Some(timestamp + lifetime_seconds),
            iss,
            aud,
            typ,
            azp,
            email,
            scope,
            client_id: None,
            sid: None,
            additional_claims: HashMap::new(),
        }
    }

    pub fn new_refresh_token(
        sub: Uuid,
        iss: String,
        aud: Vec<String>,
        azp: String,
        scope: Option<String>,
        lifetime_seconds: i64,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            sub,
            iat: timestamp,
            jti: Uuid::new_v4(),
            iss,
            aud,
            typ: ClaimsTyp::Refresh,
            azp,
            scope,
            preferred_username: None,
            email: None,
            exp: Some(timestamp + lifetime_seconds),
            client_id: None,
            sid: None,
            additional_claims: HashMap::new(),
        }
    }

    pub fn new_temporary_token(claims: JwtClaim, lifetime_seconds: i64) -> Self {
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
            exp: Some(chrono::Utc::now().timestamp() + lifetime_seconds),
            client_id: claims.client_id,
            sid: claims.sid,
            additional_claims: claims.additional_claims,
        }
    }

    /// Bind these claims to the SSO session they were issued against, so that
    /// introspection and refresh can check the session is still alive.
    pub fn with_session(mut self, sid: Uuid) -> Self {
        self.sid = Some(sid);
        self
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

pub const CERTIFICATE_VALIDITY_YEARS: i32 = 10;

#[derive(Clone)]
pub struct JwtKeyPair {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub private_key: String,
    pub public_key: String,
    pub certificate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct JwkKey {
    pub kid: String,
    pub kty: String,
    pub r#use: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

/// Lifecycle status for a persisted refresh token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshTokenStatus {
    /// Token is valid and may be exchanged exactly once.
    Active,
    /// Token has been rotated; a successor exists.
    Rotated,
    /// Token has been explicitly revoked (logout or reuse detection).
    Revoked,
}

impl RefreshTokenStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Rotated => "rotated",
            Self::Revoked => "revoked",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "rotated" => Self::Rotated,
            "revoked" => Self::Revoked,
            _ => Self::Active,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

pub struct RefreshToken {
    pub id: Uuid,
    pub jti: Uuid,
    pub user_id: Uuid,
    /// Legacy boolean; superseded by `status` but kept for backward compat with logout.
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Shared across all tokens in one rotation lineage.
    pub family_id: Uuid,
    /// Token lifecycle state.
    pub status: RefreshTokenStatus,
    /// Id of the token that superseded this one (diagnostics).
    pub replaced_by: Option<Uuid>,
    /// When this token was rotated (diagnostics).
    pub rotated_at: Option<DateTime<Utc>>,
    pub resource: Option<String>,
}

impl RefreshToken {
    #[allow(clippy::too_many_arguments)]
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
            family_id: Uuid::new_v4(),
            status: if revoked {
                RefreshTokenStatus::Revoked
            } else {
                RefreshTokenStatus::Active
            },
            replaced_by: None,
            rotated_at: None,
            resource: None,
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

impl RealmOwned for AccessToken {
    fn realm_id(&self) -> RealmId {
        RealmId::new(self.realm_id)
    }
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
        certificate_pem: &str,
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
            private_key: private_pem.to_string(),
            public_key: public_pem.to_string(),
            certificate: certificate_pem.to_string(),
        })
    }

    pub fn self_signed_certificate(
        private_pem: &str,
        common_name: &str,
    ) -> Result<String, SecurityError> {
        let signing_key = CertificateKeyPair::from_pem(private_pem)
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        let mut params = CertificateParams::new(vec![common_name.to_string()])
            .or_else(|_| CertificateParams::new(Vec::new()))
            .map_err(|e| SecurityError::GenerationError(e.to_string()))?;

        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, common_name);
        params.distinguished_name = distinguished_name;

        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];

        let current_year = Utc::now().year();
        params.not_before = date_time_ymd(current_year - 1, 1, 1);
        params.not_after = date_time_ymd(current_year + CERTIFICATE_VALIDITY_YEARS, 1, 1);

        let certificate = params
            .self_signed(&signing_key)
            .map_err(|e| SecurityError::GenerationError(e.to_string()))?;

        Ok(certificate.pem())
    }

    pub fn certificate_base64_der(&self) -> Result<String, SecurityError> {
        let body = self
            .certificate
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();

        BASE64_STANDARD
            .decode(&body)
            .map_err(|e| SecurityError::ParsingError(e.to_string()))?;

        Ok(body)
    }

    pub fn generate() -> Result<(String, String), SecurityError> {
        let key_pair = KeyPair::generate(KeySize::Rsa2048)
            .map_err(|e| SecurityError::GenerationError(e.to_string()))?;

        let private_der: Pkcs8V1Der = key_pair
            .as_der()
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;
        let public_der: PublicKeyX509Der = key_pair
            .public_key()
            .as_der()
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        Ok((
            Self::armor("PRIVATE KEY", private_der.as_ref()),
            Self::armor("PUBLIC KEY", public_der.as_ref()),
        ))
    }

    fn armor(tag: &str, der: &[u8]) -> String {
        pem::encode_config(
            &Pem::new(tag, der),
            EncodeConfig::new().set_line_ending(LineEnding::LF),
        )
    }

    pub fn to_jwk_key(&self) -> Result<JwkKey, SecurityError> {
        let armored =
            pem::parse(&self.public_key).map_err(|e| SecurityError::InvalidKey(e.to_string()))?;
        let public_key = PublicKey::from_der(armored.contents())
            .map_err(|e| SecurityError::InvalidKey(e.to_string()))?;

        let n =
            BASE64_URL_SAFE_NO_PAD.encode(public_key.modulus().big_endian_without_leading_zero());
        let e =
            BASE64_URL_SAFE_NO_PAD.encode(public_key.exponent().big_endian_without_leading_zero());

        Ok(JwkKey {
            kid: self.id.to_string(),
            kty: "RSA".to_string(),
            r#use: "sig".to_string(),
            alg: "RS256".to_string(),
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
    use super::*;
    use uuid::Uuid;
    use x509_parser::pem::parse_x509_pem;

    #[test]
    fn certificate_binds_the_existing_realm_key() {
        let (private_pem, public_pem) = JwtKeyPair::generate().expect("generate realm key");
        let certificate_pem = JwtKeyPair::self_signed_certificate(&private_pem, "master")
            .expect("derive certificate");

        let (_, pem) =
            parse_x509_pem(certificate_pem.as_bytes()).expect("certificate is valid PEM");
        assert_eq!(pem.label, "CERTIFICATE");

        let certificate = pem.parse_x509().expect("certificate is valid X.509");

        let expected_spki = pem::parse(&public_pem).expect("realm public key");

        assert_eq!(certificate.public_key().raw, expected_spki.contents());
    }

    #[test]
    fn certificate_is_self_issued_to_the_realm_and_currently_valid() {
        let (private_pem, _) = JwtKeyPair::generate().expect("generate realm key");
        let certificate_pem = JwtKeyPair::self_signed_certificate(&private_pem, "acme-realm")
            .expect("derive certificate");

        let (_, pem) =
            parse_x509_pem(certificate_pem.as_bytes()).expect("certificate is valid PEM");
        let certificate = pem.parse_x509().expect("certificate is valid X.509");

        assert_eq!(certificate.subject().to_string(), "CN=acme-realm");
        assert_eq!(
            certificate.subject().to_string(),
            certificate.issuer().to_string()
        );
        assert!(certificate.validity().is_valid());
    }

    const STORED_PRIVATE_KEY: &str = include_str!("../../tests/fixtures/realm-key.pem");
    const STORED_PUBLIC_KEY: &str = include_str!("../../tests/fixtures/realm-key.pub.pem");
    const STORED_CERTIFICATE: &str = include_str!("../../tests/fixtures/realm-cert.pem");

    fn stored_key_pair() -> JwtKeyPair {
        JwtKeyPair::from_pem(
            STORED_PRIVATE_KEY,
            STORED_PUBLIC_KEY,
            STORED_CERTIFICATE,
            Uuid::nil(),
            Uuid::nil(),
        )
        .expect("a stored realm key still loads")
    }

    #[test]
    fn a_stored_realm_key_signs_the_token_it_signed_before() {
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let claims = serde_json::json!({
            "exp": 4102444800u64,
            "iss": "https://auth.example.com/realms/master",
            "sub": "alice",
        });

        let token = jsonwebtoken::encode(&header, &claims, &stored_key_pair().encoding_key)
            .expect("sign with the stored key");

        assert_eq!(
            token,
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJleHAiOjQxMDI0NDQ4MDAsImlzcyI6Imh0dHBzOi8vYXV0aC5leGFtcGxlLmNvbS9yZWFsbXMvbWFzdGVyIiwic3ViIjoiYWxpY2UifQ.MTgaSkhCQu0LKzaR79uwNgROk4kMMYpyP4CxKQ0o3pr4qJTVjgu619TEvUJOuVKFyIZ47s787Zw7zgmYj_JVsemvlYg8hC82b5fLuBq45ZbZAhlIaLNQbBx7a1qBRVv2e1vV9q3qpGGcekVUO0Ut8YEwag2BnFZ7XbhjAAJBr8p4M9RyFxh6GOf9kjkFdp7GunVik4sELqB712eHisT1flditLsHe7GHORLZEGV2HN9xlAgA-NJMBLn0yAOX7A69AoMOjybrm0IDUNS69pZ64Tof-ndr7iKG6lSjIwDNlJEfcyOKeiWPUUHedRQHrWoK640Sy94wNxxoprPk6HDhVA"
        );
    }

    #[test]
    fn a_stored_realm_key_publishes_the_jwk_it_published_before() {
        let jwk = stored_key_pair()
            .to_jwk_key()
            .expect("publish the stored key");

        assert_eq!(
            jwk.n,
            "0B174WwrsNtE6SZImKNruqeXnazXAtKj_3g88cdOJ0EYNAVwCgpxjkZyTf8iEM8nJbPiwhjSr1RreDOGAPtsF39rzX7WBsfZEWomZfOroipgHWF4-3-3PGORYuaraVeoFDEZyWlA8pM7GGmEOyH2iygQ9CG6XzDBDrpw7wvsmjQLDnYeSKU8QlrfXD92rOpRkEbgh5qKUxVgBakMdm6qU4d3hP_OKHZ7xnyTehJCwpwfVUCvjvpolEPMEVjiynIX3_2-Ki0aXCb4eCn7MOCwNZN0EfpPJu-hYTe-0Gzvc4ZohPoCqp7W47kfd6FhtaVI7wyONnBq_rzM6ZJYMlm84Q"
        );
        assert_eq!(jwk.e, "AQAB");
    }

    #[test]
    fn certificate_survives_a_realm_name_that_is_not_a_dns_name() {
        let (private_pem, _) = JwtKeyPair::generate().expect("generate realm key");
        let certificate_pem = JwtKeyPair::self_signed_certificate(&private_pem, "réalm dé test")
            .expect("derive certificate");

        let (_, pem) =
            parse_x509_pem(certificate_pem.as_bytes()).expect("certificate is valid PEM");
        let certificate = pem.parse_x509().expect("certificate is valid X.509");

        assert_eq!(certificate.subject().to_string(), "CN=réalm dé test");
    }

    #[test]
    fn key_pair_carries_its_private_key_and_certificate() {
        let (private_pem, public_pem) = JwtKeyPair::generate().expect("generate realm key");
        let certificate_pem = JwtKeyPair::self_signed_certificate(&private_pem, "master")
            .expect("derive certificate");

        let key_pair = JwtKeyPair::from_pem(
            &private_pem,
            &public_pem,
            &certificate_pem,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .expect("build key pair");

        assert_eq!(key_pair.private_key, private_pem);
        assert_eq!(key_pair.public_key, public_pem);
        assert_eq!(key_pair.certificate, certificate_pem);
    }

    #[test]
    fn certificate_der_is_the_unarmored_base64_body() {
        let (private_pem, public_pem) = JwtKeyPair::generate().expect("generate realm key");
        let certificate_pem = JwtKeyPair::self_signed_certificate(&private_pem, "master")
            .expect("derive certificate");

        let key_pair = JwtKeyPair::from_pem(
            &private_pem,
            &public_pem,
            &certificate_pem,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .expect("build key pair");

        let encoded = key_pair
            .certificate_base64_der()
            .expect("encode certificate for metadata");

        assert!(!encoded.contains("-----"));
        assert!(!encoded.contains('\n'));

        let (_, pem) =
            parse_x509_pem(certificate_pem.as_bytes()).expect("certificate is valid PEM");
        assert_eq!(
            BASE64_STANDARD.decode(&encoded).expect("decode"),
            pem.contents
        );
    }

    #[test]
    fn refresh_claims_keep_scope() {
        let scope = Some("openid profile email".to_string());
        let claims = JwtClaim::new_refresh_token(
            Uuid::new_v4(),
            "https://issuer".to_string(),
            vec!["test-realm".to_string()],
            "client-id".to_string(),
            scope.clone(),
            DEFAULT_REFRESH_TOKEN_LIFETIME,
        );

        assert_eq!(claims.typ, ClaimsTyp::Refresh);
        assert_eq!(claims.scope, scope);
    }

    fn sample_claims() -> JwtClaim {
        JwtClaim::new(
            Uuid::new_v4(),
            "alice".to_string(),
            "https://issuer".to_string(),
            vec!["test-realm".to_string()],
            ClaimsTyp::Bearer,
            "client-id".to_string(),
            None,
            None,
            DEFAULT_ACCESS_TOKEN_LIFETIME,
        )
    }

    #[test]
    fn claims_carry_no_session_by_default() {
        assert!(sample_claims().sid.is_none());
        assert!(
            JwtClaim::new_refresh_token(
                Uuid::new_v4(),
                "https://issuer".to_string(),
                vec!["test-realm".to_string()],
                "client-id".to_string(),
                None,
                DEFAULT_REFRESH_TOKEN_LIFETIME,
            )
            .sid
            .is_none()
        );
    }

    /// `sid` sits next to a `#[serde(flatten)]` field, so a mistake in its
    /// declaration would silently route it into `additional_claims` instead of
    /// becoming a real claim. Pin both directions.
    #[test]
    fn session_id_round_trips_as_a_top_level_claim() {
        let sid = Uuid::new_v4();
        let claims = sample_claims().with_session(sid);

        let encoded = serde_json::to_value(&claims).expect("serialize claims");
        assert_eq!(
            encoded.get("sid").and_then(|v| v.as_str()),
            Some(sid.to_string().as_str())
        );

        let decoded: JwtClaim = serde_json::from_value(encoded).expect("deserialize claims");
        assert_eq!(decoded.sid, Some(sid));
        assert!(
            !decoded.additional_claims.contains_key("sid"),
            "`sid` must be a declared claim, not swept into additional_claims"
        );
    }

    /// Tokens minted before `sid` existed, and tokens from flows that establish
    /// no session, must keep deserializing and must not emit an empty `sid`.
    #[test]
    fn absent_session_id_is_omitted_and_tolerated() {
        let encoded = serde_json::to_value(sample_claims()).expect("serialize claims");
        assert!(encoded.get("sid").is_none());

        let decoded: JwtClaim = serde_json::from_value(encoded).expect("deserialize claims");
        assert!(decoded.sid.is_none());
    }

    #[test]
    fn temporary_token_keeps_its_session_binding() {
        let sid = Uuid::new_v4();
        let claims = sample_claims().with_session(sid);

        let temporary = JwtClaim::new_temporary_token(claims, DEFAULT_TEMPORARY_TOKEN_LIFETIME);

        assert_eq!(temporary.typ, ClaimsTyp::Temporary);
        assert_eq!(temporary.sid, Some(sid));
    }

    #[test]
    fn refresh_token_status_parse_roundtrip() {
        assert_eq!(
            RefreshTokenStatus::parse("active"),
            RefreshTokenStatus::Active
        );
        assert_eq!(
            RefreshTokenStatus::parse("rotated"),
            RefreshTokenStatus::Rotated
        );
        assert_eq!(
            RefreshTokenStatus::parse("revoked"),
            RefreshTokenStatus::Revoked
        );
        assert_eq!(
            RefreshTokenStatus::parse("unknown"),
            RefreshTokenStatus::Active
        );
    }

    #[test]
    fn refresh_token_status_as_str() {
        assert_eq!(RefreshTokenStatus::Active.as_str(), "active");
        assert_eq!(RefreshTokenStatus::Rotated.as_str(), "rotated");
        assert_eq!(RefreshTokenStatus::Revoked.as_str(), "revoked");
    }

    #[test]
    fn refresh_token_status_is_active() {
        assert!(RefreshTokenStatus::Active.is_active());
        assert!(!RefreshTokenStatus::Rotated.is_active());
        assert!(!RefreshTokenStatus::Revoked.is_active());
    }

    #[test]
    fn refresh_token_new_defaults_active() {
        use chrono::Utc;
        let id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let token = RefreshToken::new(id, jti, user_id, false, None, now);
        assert_eq!(token.status, RefreshTokenStatus::Active);
        assert!(!token.revoked);
    }

    #[test]
    fn refresh_token_new_revoked_reflects_status() {
        use chrono::Utc;
        let id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let token = RefreshToken::new(id, jti, user_id, true, None, now);
        assert_eq!(token.status, RefreshTokenStatus::Revoked);
        assert!(token.revoked);
    }
}
