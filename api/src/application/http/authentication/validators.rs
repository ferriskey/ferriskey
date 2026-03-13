use ferriskey_core::domain::authentication::entities::GrantType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

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

    #[validate(length(min = 43, max = 128, message = "code_challenge must be between 43 and 128 characters"))]
    #[serde(default)]
    pub code_challenge: Option<String>,

    #[serde(default)]
    pub code_challenge_method: Option<String>,

    // Space-delimited list of scopes requested
    // Example: "openid profile email"
    #[serde(default)]
    pub scope: Option<String>,
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

    #[validate(length(min = 43, max = 128, message = "code_verifier must be between 43 and 128 characters"))]
    #[serde(default)]
    pub code_verifier: Option<String>,

    #[serde(default)]
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
