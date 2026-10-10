use ferriskey_core::domain::authentication::entities::GrantType;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct TokenRequestValidator {
    #[serde(default)]
    pub grant_type: GrantType,

    // Used by `client_secret_post`
    #[serde(default)]
    pub client_id: Option<String>,

    // Used by `client_secret_post`
    #[serde(default)]
    pub client_secret: Option<String>,

    #[serde(default)]
    pub code: Option<String>,

    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(default)]
    pub refresh_token: Option<String>,

    // Scace-delimited list of scopes requested
    // Example: "openid profile email"
    #[serde(default)]
    pub scope: Option<String>,

    // Used by the device_code grant (RFC 8628 §3.4)
    #[serde(default)]
    pub device_code: Option<String>,

    // PKCE verifier (RFC 7636 §4.5)
    #[serde(default)]
    pub code_verifier: Option<String>,

    // Required by the authorization_code grant (RFC 6749 §4.1.3), where it must
    // match the redirect_uri of the originating authorization request.
    #[serde(default)]
    pub redirect_uri: Option<String>,

    // Token exchange (RFC 8693 §2.1): the token being exchanged.
    #[serde(default)]
    pub subject_token: Option<String>,

    // Token-type URN of `subject_token`. Only
    // `urn:ietf:params:oauth:token-type:access_token` is supported.
    #[serde(default)]
    pub subject_token_type: Option<String>,

    // Token-type URN wanted for the issued token. Defaults to an access token.
    #[serde(default)]
    pub requested_token_type: Option<String>,

    // `client_id` of the client the issued token is meant for.
    #[serde(default)]
    pub audience: Option<String>,

    // Target resource URI. Parsed but not supported yet (`invalid_target`).
    #[serde(default)]
    pub resource: Option<String>,

    // Token exchange delegation (RFC 8693 §2.1): the token of the party acting
    // for the subject. Its `azp` must be the requesting client.
    #[serde(default)]
    pub actor_token: Option<String>,

    // Token-type URN of `actor_token`, required with it. Only
    // `urn:ietf:params:oauth:token-type:access_token` is supported.
    #[serde(default)]
    pub actor_token_type: Option<String>,
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

    #[serde(default)]
    pub client_id: Option<String>,

    #[serde(default)]
    pub client_secret: Option<String>,

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
