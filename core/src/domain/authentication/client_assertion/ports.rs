use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::JwkSet;
use uuid::Uuid;

use crate::domain::authentication::client_assertion::entities::ClientAssertionError;
use crate::domain::authentication::client_authentication::{ClientAssertion, ClientProof};
use crate::domain::client::entities::Client;
use crate::domain::common::entities::app_errors::CoreError;

/// Reads the key set a client publishes at its `jwks_uri`.
#[cfg_attr(test, mockall::automock)]
pub trait ClientJwksFetcher: Send + Sync {
    /// `refresh` asks for a fresh copy, for an assertion signed with a key the
    /// cached copy does not have. An adapter may still answer from its cache
    /// when it refetched very recently.
    fn fetch(
        &self,
        uri: &str,
        refresh: bool,
    ) -> impl Future<Output = Result<JwkSet, ClientAssertionError>> + Send;
}

/// Remembers the `jti` of every accepted assertion until it expires.
#[cfg_attr(test, mockall::automock)]
pub trait ClientAssertionReplayStore: Send + Sync {
    /// `false` when this client already used `jti` and it has not expired.
    fn record(
        &self,
        client_id: Uuid,
        jti: &str,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn delete_expired(
        &self,
        before: DateTime<Utc>,
        limit: u64,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

/// Checks a client assertion (RFC 7523 §3) against the client it names.
#[cfg_attr(test, mockall::automock)]
pub trait ClientAssertionVerifier: Send + Sync {
    fn verify(
        &self,
        client: &Client,
        assertion: &ClientAssertion,
    ) -> impl Future<Output = Result<ClientProof, CoreError>> + Send;
}
