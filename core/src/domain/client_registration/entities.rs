use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

pub const MAX_REDIRECT_URIS: usize = 10;
pub const MAX_REDIRECT_URI_BYTES: usize = 2048;
pub const MAX_DYNAMIC_CLIENTS_PER_REALM: u64 = 1000;
pub const MAX_CLIENT_NAME_CHARS: usize = 128;
pub const DEFAULT_CLIENT_NAME: &str = "Dynamic client";
pub const AUTH_METHOD_NONE: &str = "none";
pub const AUTH_METHOD_BASIC: &str = "client_secret_basic";
pub const AUTH_METHOD_POST: &str = "client_secret_post";
pub const GRANT_AUTHORIZATION_CODE: &str = "authorization_code";
pub const GRANT_REFRESH_TOKEN: &str = "refresh_token";
pub const RESPONSE_TYPE_CODE: &str = "code";

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, ToSchema)]
pub struct ClientRegistrationRequest {
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    pub grant_types: Option<Vec<String>>,
    #[serde(default)]
    pub response_types: Option<Vec<String>>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_id_issued_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_expires_at: Option<i64>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedRegistration {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub public_client: bool,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistrationError {
    #[error("dynamic client registration is not available")]
    Disabled,

    #[error("too many registration requests")]
    RateLimited,

    #[error("invalid_redirect_uri: {0}")]
    InvalidRedirectUri(&'static str),

    #[error("invalid_client_metadata: {0}")]
    InvalidClientMetadata(&'static str),

    #[error("the client could not be registered")]
    Internal,
}
