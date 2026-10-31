use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

/// How a client authenticates at the token, introspection and revocation
/// endpoints (RFC 7591 §2, OIDC Core §9).
///
/// `client_secret_basic` and `client_secret_post` differ only in where the
/// secret travels; either transport is accepted for both.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TokenEndpointAuthMethod {
    None,
    #[default]
    ClientSecretBasic,
    ClientSecretPost,
    ClientSecretJwt,
    PrivateKeyJwt,
}

impl TokenEndpointAuthMethod {
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::ClientSecretBasic,
        Self::ClientSecretPost,
        Self::ClientSecretJwt,
        Self::PrivateKeyJwt,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ClientSecretBasic => "client_secret_basic",
            Self::ClientSecretPost => "client_secret_post",
            Self::ClientSecretJwt => "client_secret_jwt",
            Self::PrivateKeyJwt => "private_key_jwt",
        }
    }

    pub fn is_public(self) -> bool {
        self == Self::None
    }

    pub fn uses_client_keys(self) -> bool {
        self == Self::PrivateKeyJwt
    }

    pub fn needs_secret(self) -> bool {
        matches!(
            self,
            Self::ClientSecretBasic | Self::ClientSecretPost | Self::ClientSecretJwt
        )
    }

    pub fn default_for(public_client: bool) -> Self {
        if public_client {
            Self::None
        } else {
            Self::ClientSecretBasic
        }
    }
}

impl fmt::Display for TokenEndpointAuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown token_endpoint_auth_method: {0}")]
pub struct UnknownAuthMethod(pub String);

impl FromStr for TokenEndpointAuthMethod {
    type Err = UnknownAuthMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|method| method.as_str() == s)
            .ok_or_else(|| UnknownAuthMethod(s.to_string()))
    }
}

/// An inline JWK set (RFC 7517 §5), kept as the client sent it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = Object)]
pub struct ClientJwks(pub serde_json::Value);

impl PartialOrd for ClientJwks {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClientJwks {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.to_string().cmp(&other.0.to_string())
    }
}

/// Where the public keys of a `private_key_jwt` client come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKeySource<'a> {
    Inline(&'a ClientJwks),
    Uri(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidClientAuth {
    #[error("a public client uses none, a confidential client cannot")]
    MethodMismatch,
    #[error("jwks and jwks_uri cannot both be set")]
    BothKeySources,
    #[error("private_key_jwt needs jwks or jwks_uri")]
    MissingKeys,
    #[error("jwks is not a JWK set: {0}")]
    MalformedKeySet(&'static str),
    #[error("jwks must not contain private or symmetric keys")]
    SecretKeyMaterial,
    #[error("jwks_uri must be an http(s) url without credentials")]
    InvalidJwksUri,
}

impl From<InvalidClientAuth> for crate::common::app_errors::CoreError {
    fn from(error: InvalidClientAuth) -> Self {
        Self::InvalidClientAuthentication(error.to_string())
    }
}

const PRIVATE_JWK_MEMBERS: [&str; 7] = ["d", "p", "q", "dp", "dq", "qi", "oth"];

/// The client authentication part of a client: its method and, for
/// `private_key_jwt`, where its public keys are.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientAuthSettings {
    pub method: TokenEndpointAuthMethod,
    pub jwks: Option<ClientJwks>,
    pub jwks_uri: Option<String>,
}

impl ClientAuthSettings {
    pub fn default_for(public_client: bool) -> Self {
        Self {
            method: TokenEndpointAuthMethod::default_for(public_client),
            jwks: None,
            jwks_uri: None,
        }
    }

    /// Checks the settings against the client they belong to.
    ///
    /// `none` is the method of public clients and only theirs. Only a
    /// `private_key_jwt` client needs keys, and exactly one source of them.
    /// An inline set holds public material only: a private or symmetric key
    /// stored here would be readable by every admin.
    pub fn validate(&self, public_client: bool) -> Result<(), InvalidClientAuth> {
        if self.method.is_public() != public_client {
            return Err(InvalidClientAuth::MethodMismatch);
        }

        if self.jwks.is_some() && self.jwks_uri.is_some() {
            return Err(InvalidClientAuth::BothKeySources);
        }

        if self.method.uses_client_keys() && self.jwks.is_none() && self.jwks_uri.is_none() {
            return Err(InvalidClientAuth::MissingKeys);
        }

        if let Some(uri) = &self.jwks_uri {
            validate_jwks_uri(uri)?;
        }

        match &self.jwks {
            Some(jwks) => validate_public_key_set(&jwks.0),
            None => Ok(()),
        }
    }
}

/// Structure only: which hosts may be reached is decided when the keys are
/// fetched, against the address the host resolves to then.
fn validate_jwks_uri(raw: &str) -> Result<(), InvalidClientAuth> {
    let url = url::Url::parse(raw).map_err(|_| InvalidClientAuth::InvalidJwksUri)?;
    let structured = matches!(url.scheme(), "https" | "http")
        && url.host().is_some()
        && url.username().is_empty()
        && url.password().is_none();

    if structured {
        Ok(())
    } else {
        Err(InvalidClientAuth::InvalidJwksUri)
    }
}

fn validate_public_key_set(jwks: &serde_json::Value) -> Result<(), InvalidClientAuth> {
    let keys = jwks
        .get("keys")
        .and_then(serde_json::Value::as_array)
        .ok_or(InvalidClientAuth::MalformedKeySet("keys must be an array"))?;

    if keys.is_empty() {
        return Err(InvalidClientAuth::MalformedKeySet("keys is empty"));
    }

    for key in keys {
        let key = key
            .as_object()
            .ok_or(InvalidClientAuth::MalformedKeySet("a key is not an object"))?;

        match key.get("kty").and_then(serde_json::Value::as_str) {
            Some("RSA" | "EC" | "OKP") => {}
            Some("oct") => return Err(InvalidClientAuth::SecretKeyMaterial),
            _ => {
                return Err(InvalidClientAuth::MalformedKeySet(
                    "a key has no usable kty",
                ));
            }
        }

        if PRIVATE_JWK_MEMBERS
            .iter()
            .any(|member| key.contains_key(*member))
        {
            return Err(InvalidClientAuth::SecretKeyMaterial);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn rsa_public() -> serde_json::Value {
        json!({ "keys": [{ "kty": "RSA", "kid": "k1", "n": "abc", "e": "AQAB" }] })
    }

    #[test]
    fn every_method_round_trips_through_its_name() {
        for method in TokenEndpointAuthMethod::ALL {
            assert_eq!(method.as_str().parse(), Ok(method));
            assert_eq!(
                serde_json::to_value(method).expect("serialize"),
                json!(method.as_str())
            );
        }
    }

    #[test]
    fn an_unknown_name_is_refused() {
        assert!(
            "tls_client_auth"
                .parse::<TokenEndpointAuthMethod>()
                .is_err()
        );
    }

    fn settings(
        method: TokenEndpointAuthMethod,
        jwks: Option<serde_json::Value>,
        jwks_uri: Option<&str>,
    ) -> ClientAuthSettings {
        ClientAuthSettings {
            method,
            jwks: jwks.map(ClientJwks),
            jwks_uri: jwks_uri.map(str::to_string),
        }
    }

    const JWKS_URI: Option<&str> = Some("https://app.example/jwks.json");

    #[test]
    fn none_belongs_to_public_clients_only() {
        assert!(ClientAuthSettings::default_for(true).validate(true).is_ok());
        assert!(
            ClientAuthSettings::default_for(false)
                .validate(false)
                .is_ok()
        );
        assert_eq!(
            ClientAuthSettings::default_for(true).validate(false),
            Err(InvalidClientAuth::MethodMismatch)
        );
        assert_eq!(
            ClientAuthSettings::default_for(false).validate(true),
            Err(InvalidClientAuth::MethodMismatch)
        );
    }

    #[test]
    fn private_key_jwt_needs_a_key_source() {
        use TokenEndpointAuthMethod::PrivateKeyJwt;
        assert_eq!(
            settings(PrivateKeyJwt, None, None).validate(false),
            Err(InvalidClientAuth::MissingKeys)
        );
        assert!(
            settings(PrivateKeyJwt, None, JWKS_URI)
                .validate(false)
                .is_ok()
        );
        assert!(
            settings(PrivateKeyJwt, Some(rsa_public()), None)
                .validate(false)
                .is_ok()
        );
    }

    #[test]
    fn both_key_sources_are_refused() {
        assert_eq!(
            settings(
                TokenEndpointAuthMethod::PrivateKeyJwt,
                Some(rsa_public()),
                JWKS_URI
            )
            .validate(false),
            Err(InvalidClientAuth::BothKeySources)
        );
    }

    #[test]
    fn secret_methods_need_no_keys() {
        for method in [
            TokenEndpointAuthMethod::ClientSecretBasic,
            TokenEndpointAuthMethod::ClientSecretPost,
            TokenEndpointAuthMethod::ClientSecretJwt,
        ] {
            assert!(
                settings(method, None, None).validate(false).is_ok(),
                "{method}"
            );
        }
    }

    #[test]
    fn private_key_material_is_refused() {
        use TokenEndpointAuthMethod::PrivateKeyJwt;
        let leaked = json!({ "keys": [{ "kty": "RSA", "n": "abc", "e": "AQAB", "d": "secret" }] });
        assert_eq!(
            settings(PrivateKeyJwt, Some(leaked), None).validate(false),
            Err(InvalidClientAuth::SecretKeyMaterial)
        );

        let symmetric = json!({ "keys": [{ "kty": "oct", "k": "c2VjcmV0" }] });
        assert_eq!(
            settings(PrivateKeyJwt, Some(symmetric), None).validate(false),
            Err(InvalidClientAuth::SecretKeyMaterial)
        );
    }

    #[test]
    fn a_jwks_uri_must_be_a_plain_http_url() {
        use TokenEndpointAuthMethod::PrivateKeyJwt;
        for uri in [
            "not a url",
            "file:///etc/jwks.json",
            "https://user:pw@app.example/jwks.json",
        ] {
            assert_eq!(
                settings(PrivateKeyJwt, None, Some(uri)).validate(false),
                Err(InvalidClientAuth::InvalidJwksUri),
                "{uri}"
            );
        }
    }

    #[test]
    fn a_set_without_keys_is_refused() {
        for jwks in [json!({}), json!({ "keys": [] }), json!({ "keys": [{}] })] {
            assert!(matches!(
                settings(TokenEndpointAuthMethod::PrivateKeyJwt, Some(jwks), None).validate(false),
                Err(InvalidClientAuth::MalformedKeySet(_))
            ));
        }
    }
}
