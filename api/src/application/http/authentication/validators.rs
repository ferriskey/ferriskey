use ferriskey_core::domain::authentication::entities::GrantType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

static CODE_VERIFIER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9\-._~]+$").unwrap());

/// RFC 7636 code_challenge characters: base64url alphabet without padding
/// Allows: A-Z, a-z, 0-9, hyphen, underscore
static CODE_CHALLENGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9\-_]+$").unwrap());

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct AuthRequestValidator {
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,

    #[serde(default)]
    pub redirect_uri: Option<String>,

    #[serde(default)]
    pub response_type: Option<String>,

    #[serde(default)]
    pub state: Option<String>,

    #[serde(default)]
    pub nonce: Option<String>,

    #[validate(length(min = 43, max = 128, message = "code_challenge must be between 43 and 128 characters"), regex(path = *CODE_CHALLENGE_RE, message = "code_challenge must contain only base64url characters (A-Z, a-z, 0-9, hyphen, underscore)"))]
    #[serde(default)]
    pub code_challenge: Option<String>,

    #[serde(default)]
    pub code_challenge_method: Option<CodeChallengeMethod>,

    // Space-delimited list of scopes requested
    // Example: "openid profile email"
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
pub enum CodeChallengeMethod {
    #[serde(rename = "S256")]
    S256,
}

impl Default for CodeChallengeMethod {
    fn default() -> Self {
        CodeChallengeMethod::S256
    }
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct TokenRequestValidator {
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,

    #[serde(default)]
    pub client_secret: Option<String>,

    #[serde(default)]
    pub code: Option<String>,

    #[serde(default)]
    pub refresh_token: Option<String>,

    #[validate(length(min = 43, max = 128, message = "code_verifier must be between 43 and 128 characters"), regex(path = *CODE_VERIFIER_RE, message = "code_verifier must contain only valid characters"))]
    #[serde(default)]
    pub code_verifier: Option<String>,

    pub grant_type: GrantType,

    // Space-delimited list of scopes requested
    // Example: "openid profile email"
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct IntrospectRequestValidator {
    #[validate(length(min = 1, message = "token is required"))]
    #[serde(default)]
    pub token: String,

    #[serde(default)]
    pub token_type_hint: Option<String>,

    // Used by `client_secret_post`
    #[serde(default)]
    pub client_id: Option<String>,

    // Used by `client_secret_post`
    #[serde(default)]
    pub client_secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RevokeTokenRequestValidator {
    #[validate(length(min = 1, message = "token is required"))]
    #[serde(default)]
    pub token: String,

    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,

    #[serde(default)]
    pub token_type_hint: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogoutRequestValidator {
    #[serde(default)]
    pub id_token_hint: Option<String>,

    #[serde(default)]
    pub post_logout_redirect_uri: Option<String>,

    #[serde(default)]
    pub state: Option<String>,

    #[serde(default)]
    pub client_id: Option<String>,
}
