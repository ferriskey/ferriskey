use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::authentication::client_authentication::ClientCredential;
use crate::domain::authentication::value_objects::CodeChallengeMethod;
use crate::domain::realm::entities::RealmId;

pub const REQUEST_URI_PREFIX: &str = "urn:ietf:params:oauth:request_uri:";

/// How long a pushed request waits for `/auth` (RFC 9126 §2.2 suggests
/// between 5 and 600 seconds).
pub const PUSHED_REQUEST_LIFETIME_SECS: i64 = 60;

const REFERENCE_BYTES: usize = 32;

/// The authorization request parameters a client pushed, replayed by `/auth`
/// in place of its query string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationParameters {
    pub response_type: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
    pub prompt: Option<String>,
    pub max_age: Option<i64>,
    pub resource: Option<String>,
}

/// A `urn:ietf:params:oauth:request_uri:<reference>` handed to the client.
///
/// Only its digest is stored, so a read of the table cannot be replayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestUri(String);

impl RequestUri {
    pub fn generate() -> Self {
        let mut reference = [0u8; REFERENCE_BYTES];
        rand::rng().fill(&mut reference[..]);
        Self(format!(
            "{REQUEST_URI_PREFIX}{}",
            URL_SAFE_NO_PAD.encode(reference)
        ))
    }

    pub fn parse(value: &str) -> Option<Self> {
        let reference = value.strip_prefix(REQUEST_URI_PREFIX)?;
        let well_formed = !reference.is_empty()
            && reference.len() <= 128
            && reference
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        well_formed.then(|| Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn digest(&self) -> RequestUriDigest {
        RequestUriDigest(hex::encode(Sha256::digest(self.0.as_bytes())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestUriDigest(String);

impl RequestUriDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushedAuthorizationRequest {
    pub digest: RequestUriDigest,
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub parameters: AuthorizationParameters,
    pub expires_at: DateTime<Utc>,
}

impl PushedAuthorizationRequest {
    pub fn new(
        request_uri: &RequestUri,
        realm_id: RealmId,
        client_id: Uuid,
        parameters: AuthorizationParameters,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            digest: request_uri.digest(),
            realm_id,
            client_id,
            parameters,
            expires_at: now + Duration::seconds(PUSHED_REQUEST_LIFETIME_SECS),
        }
    }
}

pub struct PushAuthorizationRequestInput {
    pub realm_name: String,
    pub client_id: String,
    pub client_credential: ClientCredential,
    pub parameters: AuthorizationParameters,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub issuer: Option<String>,
}

/// The PAR endpoint response (RFC 9126 §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,
    pub expires_in: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_request_uri_parses_back_to_itself() {
        let generated = RequestUri::generate();

        assert!(generated.as_str().starts_with(REQUEST_URI_PREFIX));
        assert_eq!(RequestUri::parse(generated.as_str()), Some(generated));
    }

    #[test]
    fn two_generated_request_uris_differ() {
        assert_ne!(RequestUri::generate(), RequestUri::generate());
    }

    #[test]
    fn a_request_uri_needs_the_urn_prefix_and_a_reference() {
        for value in [
            "",
            "abc",
            REQUEST_URI_PREFIX,
            "urn:ietf:params:oauth:request_uri",
            "https://iam.example/par/abc",
            "urn:ietf:params:oauth:request_uri:a b",
            "urn:ietf:params:oauth:request_uri:a%20b",
        ] {
            assert_eq!(RequestUri::parse(value), None, "{value:?}");
        }
    }

    #[test]
    fn an_oversized_reference_is_refused() {
        let value = format!("{REQUEST_URI_PREFIX}{}", "a".repeat(129));

        assert_eq!(RequestUri::parse(&value), None);
    }

    #[test]
    fn the_digest_is_stable_and_does_not_reveal_the_uri() {
        let request_uri = RequestUri::generate();

        assert_eq!(request_uri.digest(), request_uri.digest());
        assert!(!request_uri.digest().as_str().contains(request_uri.as_str()));
        assert_eq!(request_uri.digest().as_str().len(), 64);
    }

    #[test]
    fn a_pushed_request_expires_after_its_lifetime() {
        let now = Utc::now();
        let request = PushedAuthorizationRequest::new(
            &RequestUri::generate(),
            RealmId::from(Uuid::new_v4()),
            Uuid::new_v4(),
            AuthorizationParameters {
                response_type: "code".to_string(),
                redirect_uri: "https://app.example/cb".to_string(),
                scope: None,
                state: None,
                nonce: None,
                code_challenge: None,
                code_challenge_method: None,
                prompt: None,
                max_age: None,
                resource: None,
            },
            now,
        );

        assert_eq!(
            request.expires_at - now,
            Duration::seconds(PUSHED_REQUEST_LIFETIME_SECS)
        );
    }
}
