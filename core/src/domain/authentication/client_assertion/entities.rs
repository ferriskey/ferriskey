use std::time::Duration;

use jsonwebtoken::Algorithm;
use serde::Deserialize;
use thiserror::Error;

use crate::domain::common::entities::app_errors::CoreError;

/// Longest lifetime an assertion may claim. Its `jti` is remembered until it
/// expires, so this also bounds the replay store.
pub const MAX_ASSERTION_LIFETIME_SECONDS: i64 = 60 * 60;

pub const ASSERTION_LEEWAY_SECONDS: u64 = 30;

pub const MAX_JTI_CHARS: usize = 256;

pub const PRIVATE_KEY_JWT_ALGORITHMS: [Algorithm; 9] = [
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::PS256,
    Algorithm::PS384,
    Algorithm::PS512,
    Algorithm::ES256,
    Algorithm::ES384,
    Algorithm::EdDSA,
];

pub const CLIENT_SECRET_JWT_ALGORITHMS: [Algorithm; 3] =
    [Algorithm::HS256, Algorithm::HS384, Algorithm::HS512];

pub const JWKS_FETCH_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_JWKS_BYTES: usize = 64 * 1024;
pub const DEFAULT_JWKS_CACHE_AGE: Duration = Duration::from_secs(5 * 60);
pub const MAX_JWKS_CACHE_AGE: Duration = Duration::from_secs(24 * 60 * 60);
pub const MIN_JWKS_REFETCH_INTERVAL: Duration = Duration::from_secs(10);
pub const MAX_JWKS_CACHE_ENTRIES: usize = 1024;

/// `token_endpoint_auth_signing_alg_values_supported`, for discovery.
pub fn assertion_signing_algorithms() -> Vec<String> {
    PRIVATE_KEY_JWT_ALGORITHMS
        .iter()
        .chain(CLIENT_SECRET_JWT_ALGORITHMS.iter())
        .map(|alg| format!("{alg:?}"))
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct AssertionClaims {
    pub exp: i64,
    #[serde(default)]
    pub jti: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientAssertionError {
    #[error("the assertion is not a JWT")]
    Malformed,
    #[error("the client is not registered for this assertion method")]
    MethodMismatch,
    #[error("the assertion algorithm is not accepted for this client")]
    UnsupportedAlgorithm,
    #[error("the client has no usable key")]
    NoKeys,
    #[error("the client keys could not be fetched: {0}")]
    KeysUnavailable(&'static str),
    #[error("no client key matches the assertion")]
    UnknownKey,
    #[error("the assertion did not verify: {0}")]
    Rejected(String),
    #[error("the assertion has no usable jti")]
    MissingJti,
    #[error("the assertion lives longer than allowed")]
    TooLongLived,
    #[error("the assertion was already used")]
    Replayed,
    #[error("the replay store failed")]
    Store,
}

impl From<ClientAssertionError> for CoreError {
    fn from(error: ClientAssertionError) -> Self {
        match error {
            ClientAssertionError::Store => CoreError::InternalServerError,
            _ => CoreError::ClientAuthenticationFailed,
        }
    }
}
