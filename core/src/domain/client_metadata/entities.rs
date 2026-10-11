use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

use crate::domain::common::entities::app_errors::CoreError;

pub const MAX_CLIENT_ID_URL_LEN: usize = 255;
pub const MAX_DOCUMENT_BYTES: usize = 5 * 1024;
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_CACHE_AGE: Duration = Duration::from_secs(24 * 60 * 60);
pub const DEFAULT_CACHE_AGE: Duration = Duration::from_secs(5 * 60);
pub const FAILURE_CACHE_AGE: Duration = Duration::from_secs(60);
pub const MAX_CACHE_ENTRIES: usize = 1024;
pub const MAX_CLIENT_NAME_CHARS: usize = 128;
pub const MAX_METADATA_DOCUMENT_CLIENTS_PER_REALM: u64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ClientMetadataDocument {
    pub client_id: String,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub token_endpoint_auth_method: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedClientMetadata {
    pub document: ClientMetadataDocument,
    pub max_age: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientMetadataError {
    #[error("the client_id is not an acceptable metadata document url")]
    InvalidUrl,

    #[error("the metadata document host is not allowed in this realm")]
    HostNotAllowed,

    #[error("the metadata document could not be fetched: {0}")]
    FetchFailed(&'static str),

    #[error("the metadata document is not valid: {0}")]
    InvalidDocument(&'static str),

    #[error("the metadata document client_id does not match its url")]
    ClientIdMismatch,

    #[error("the redirect_uri is not listed in the metadata document")]
    RedirectUriNotListed,

    #[error("the metadata document asks for an unsupported client authentication method")]
    UnsupportedAuthMethod,

    #[error("too many metadata document fetches from this address")]
    RateLimited,

    #[error("this realm has reached its limit of metadata document clients")]
    RealmLimitReached,

    #[error("the client_id is already used by a client that is not a metadata document client")]
    ClientIdTaken,
}

impl From<ClientMetadataError> for CoreError {
    fn from(error: ClientMetadataError) -> Self {
        match error {
            ClientMetadataError::RedirectUriNotListed => CoreError::InvalidRedirectUri,
            _ => CoreError::InvalidClient,
        }
    }
}
