use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Query parameters for broker login endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct BrokerLoginRequest {
    /// Client ID initiating the login
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,

    /// Redirect URI to return to after authentication
    #[validate(length(min = 1, message = "redirect_uri is required"))]
    #[serde(default)]
    pub redirect_uri: String,

    /// OAuth response type (defaults to "code")
    #[serde(default = "default_response_type")]
    pub response_type: String,

    /// Requested scopes
    #[serde(default)]
    pub scope: Option<String>,

    /// Client's state parameter for CSRF protection
    #[serde(default)]
    pub state: Option<String>,

    /// OIDC nonce for replay protection
    #[serde(default)]
    pub nonce: Option<String>,

    /// Existing auth session ID (if initiated from login page)
    #[serde(default)]
    pub session_id: Option<Uuid>,
}

fn default_response_type() -> String {
    "code".to_string()
}

/// Query parameters for broker callback endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct BrokerCallbackQuery {
    /// Authorization code from IdP
    #[serde(default)]
    pub code: Option<String>,

    /// State parameter for CSRF validation (required)
    pub state: String,

    /// Error code from IdP (if authentication failed)
    #[serde(default)]
    pub error: Option<String>,

    /// Error description from IdP
    #[serde(default)]
    pub error_description: Option<String>,
}
