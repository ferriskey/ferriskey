use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::Identity;
use crate::authentication::value_objects::CodeChallengeMethod;
use crate::user::entities::RequiredAction;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct JwtToken {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u32,
    refresh_expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id_token: Option<String>,
}

impl JwtToken {
    pub fn new(
        access_token: String,
        token_type: String,
        refresh_token: String,
        expires_in: u32,
        refresh_expires_in: u32,
        session_state: Option<String>,
        id_token: Option<String>,
    ) -> Self {
        Self {
            access_token,
            token_type,
            refresh_token,
            expires_in,
            refresh_expires_in,
            session_state,
            id_token,
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
}

impl RefreshClaims {
    pub fn new(sub: String, exp: usize, jti: String) -> Self {
        Self { sub, exp, jti }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct TokenIntrospectionResponse {
    pub active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::TokenIntrospectionResponse;
    use serde_json::json;

    #[test]
    fn inactive_serializes_to_rfc7662_minimal_shape() {
        let body = TokenIntrospectionResponse {
            active: false,
            ..Default::default()
        };

        let v = serde_json::to_value(body).unwrap();
        assert_eq!(v, json!({ "active": false }));
    }
}

#[derive(Debug, Clone, Error)]
pub enum AuthenticationError {
    #[error("Token not found")]
    NotFound,

    #[error("Service account not found")]
    ServiceAccountNotFound,

    #[error("Invalid client")]
    Invalid,

    #[error("Invalid realm")]
    InvalidRealm,

    #[error("Invalid client")]
    InvalidClient,

    #[error("Invalid user")]
    InvalidUser,

    #[error("Password is invalid")]
    InvalidPassword,

    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Invalid authorization request")]
    InvalidRequest,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum GrantType {
    #[default]
    #[serde(rename = "authorization_code")]
    Code,

    #[serde(rename = "password")]
    Password,

    #[serde(rename = "client_credentials")]
    Credentials,

    #[serde(rename = "refresh_token")]
    RefreshToken,

    #[serde(rename = "urn:ietf:params:oauth:grant-type:device_code")]
    DeviceCode,

    #[serde(rename = "urn:ietf:params:oauth:grant-type:token-exchange")]
    TokenExchange,
}

impl Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantType::Code => write!(f, "code"),
            GrantType::Password => write!(f, "password"),
            GrantType::Credentials => write!(f, "credentials"),
            GrantType::RefreshToken => write!(f, "refresh_token"),
            GrantType::DeviceCode => write!(f, "urn:ietf:params:oauth:grant-type:device_code"),
            GrantType::TokenExchange => {
                write!(f, "urn:ietf:params:oauth:grant-type:token-exchange")
            }
        }
    }
}

pub const OPENID_CONNECT_PROTOCOL: &str = "openid-connect";
pub const SAML_PROTOCOL: &str = "saml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthProtocol {
    #[default]
    #[serde(rename = "openid-connect")]
    OpenIdConnect,
    #[serde(rename = "saml")]
    Saml,
}

impl AuthProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthProtocol::OpenIdConnect => OPENID_CONNECT_PROTOCOL,
            AuthProtocol::Saml => SAML_PROTOCOL,
        }
    }
}

impl Display for AuthProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuthProtocol {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            OPENID_CONNECT_PROTOCOL => Ok(AuthProtocol::OpenIdConnect),
            SAML_PROTOCOL => Ok(AuthProtocol::Saml),
            other => Err(format!("unsupported authentication protocol: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthCompletion {
    Redirect {
        url: String,
    },
    FormPost {
        action_url: String,
        body: String,
        relay_state: Option<String>,
    },
}

impl AuthCompletion {
    pub fn redirect_url(&self) -> Option<&str> {
        match self {
            AuthCompletion::Redirect { url } => Some(url),
            AuthCompletion::FormPost { .. } => None,
        }
    }

    pub fn into_redirect_url(self) -> Option<String> {
        match self {
            AuthCompletion::Redirect { url } => Some(url),
            AuthCompletion::FormPost { .. } => None,
        }
    }
}

pub struct AuthInput {
    pub client_id: String,
    pub realm_name: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

pub struct ExchangeTokenInput {
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub base_url: String,
    pub grant_type: GrantType,
    pub scope: Option<String>,
    /// Set for the `urn:ietf:params:oauth:grant-type:device_code` grant.
    pub device_code: Option<String>,
    pub code_verifier: Option<String>,
    /// REQUIRED for the `authorization_code` grant (RFC 6749 §4.1.3): must be
    /// identical to the one sent in the authorization request.
    pub redirect_uri: Option<String>,
}

pub struct AuthorizeRequestOutput {
    pub identity: Identity,
}

pub struct AuthenticateInput {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub auth_method: AuthenticationMethod,
}

impl AuthenticateInput {
    pub fn with_user_credentials(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::UserCredentials { username, password },
        }
    }

    pub fn with_existing_token(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        token: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::ExistingToken { token },
        }
    }

    pub fn is_token_refresh(&self) -> bool {
        matches!(self.auth_method, AuthenticationMethod::ExistingToken { .. })
    }

    pub fn is_credential_auth(&self) -> bool {
        matches!(
            self.auth_method,
            AuthenticationMethod::UserCredentials { .. }
        )
    }
}

pub struct AuthenticateOutput {
    pub user_id: Uuid,
    pub status: AuthenticationStepStatus,
    pub authorization_code: Option<String>,
    pub temporary_token: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub redirect_url: Option<String>,
    pub completion: Option<AuthCompletion>,
    pub session_state: Option<String>,
    pub email: Option<String>,
}

impl AuthenticateOutput {
    pub fn complete(user_id: Uuid, authorization_code: String, completion: AuthCompletion) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::Success,
            authorization_code: Some(authorization_code),
            temporary_token: None,
            required_actions: Vec::new(),
            redirect_url: completion.redirect_url().map(str::to_string),
            completion: Some(completion),
            session_state: None,
            email: None,
        }
    }

    pub fn requires_actions(
        user_id: Uuid,
        required_actions: Vec<RequiredAction>,
        temporary_token: String,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresActions,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions,
            redirect_url: None,
            completion: None,
            session_state: None,
            email: None,
        }
    }

    pub fn requires_otp_challenge(
        user_id: Uuid,
        temporary_token: String,
        email: Option<String>,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresOtpChallenge,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions: Vec::new(),
            redirect_url: None,
            completion: None,
            session_state: None,
            email,
        }
    }
}

#[derive(Debug)]
pub struct CredentialsAuthParams {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationStepStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    UserCredentials { username: String, password: String },
    ExistingToken { token: String },
}

#[cfg(test)]
mod grant_type_tests {
    use super::*;

    const TOKEN_EXCHANGE_URN: &str = "urn:ietf:params:oauth:grant-type:token-exchange";

    #[test]
    fn grant_type_token_exchange_serde_round_trips_to_exact_urn() {
        let json = format!("\"{TOKEN_EXCHANGE_URN}\"");
        let parsed: GrantType = serde_json::from_str(&json)
            .expect("token-exchange URN should deserialize into GrantType");
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }

    #[test]
    fn grant_type_token_exchange_displays_full_urn() {
        let json = format!("\"{TOKEN_EXCHANGE_URN}\"");
        let parsed: GrantType = serde_json::from_str(&json)
            .expect("token-exchange URN should deserialize into GrantType");
        assert_eq!(parsed.to_string(), TOKEN_EXCHANGE_URN);
    }
}

#[cfg(test)]
mod auth_protocol_tests {
    use super::*;

    #[test]
    fn auth_protocol_round_trips_through_the_value_stored_in_the_client_row() {
        for protocol in [AuthProtocol::OpenIdConnect, AuthProtocol::Saml] {
            assert_eq!(
                protocol.to_string().parse::<AuthProtocol>(),
                Ok(protocol),
                "{protocol} must survive a database round trip"
            );
        }
    }

    #[test]
    fn auth_protocol_defaults_to_the_value_every_existing_row_carries() {
        assert_eq!(AuthProtocol::default().as_str(), "openid-connect");
    }

    #[test]
    fn an_unknown_protocol_is_rejected_rather_than_treated_as_openid_connect() {
        assert!("ws-federation".parse::<AuthProtocol>().is_err());
    }

    #[test]
    fn only_a_redirect_completion_yields_a_redirect_url() {
        let redirect = AuthCompletion::Redirect {
            url: "https://client.example/cb?code=C".to_string(),
        };
        assert_eq!(
            redirect.redirect_url(),
            Some("https://client.example/cb?code=C")
        );

        let form_post = AuthCompletion::FormPost {
            action_url: "https://sp.example/acs".to_string(),
            body: "PHNhbWxwOlJlc3BvbnNlLz4=".to_string(),
            relay_state: Some("relay".to_string()),
        };
        assert_eq!(form_post.redirect_url(), None);
        assert_eq!(form_post.into_redirect_url(), None);
    }
}
