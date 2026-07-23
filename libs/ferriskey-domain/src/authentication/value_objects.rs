use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::authentication::entities::{GrantType, JwtToken};
use crate::realm::RealmId;
use crate::user::entities::{RequiredAction, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CodeChallengeMethod {
    S256,
    Plain,
}

impl std::fmt::Display for CodeChallengeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeChallengeMethod::S256 => write!(f, "S256"),
            CodeChallengeMethod::Plain => write!(f, "plain"),
        }
    }
}

impl std::str::FromStr for CodeChallengeMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S256" => Ok(CodeChallengeMethod::S256),
            "plain" => Ok(CodeChallengeMethod::Plain),
            other => Err(format!("unsupported code_challenge_method: {other}")),
        }
    }
}

pub struct AuthenticateRequest {
    pub realm_name: String,
    pub grant_type: GrantType,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateAuthSessionRequest {
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

pub struct GrantTypeParams {
    pub realm_id: RealmId,
    pub base_url: String,
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub code: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub user_id: Uuid,
    pub token: Option<String>,
    pub credentials: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserInput {
    pub realm_name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub session_code: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterUserUrlContext {
    pub issuer_base_url: String,
    pub verification_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum RegisterUserOutput {
    /// Normal registration - returns JWT tokens
    Authenticated(JwtToken),
    /// Registration completed inside an OIDC flow - redirect to the original client
    Redirect { url: String },
    /// Email verification required - no tokens
    PendingVerification { message: String, user_id: Uuid },
}

pub struct GenerateTokensForUserInput {
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub base_url: String,
    pub client_id: Option<Uuid>,
}

impl CreateAuthSessionRequest {
    pub fn new(realm_id: Uuid, client_id: Uuid, redirect_uri: String) -> Self {
        Self {
            realm_id,
            client_id,
            redirect_uri,
            response_type: "code".to_string(),
            scope: Some("openid".to_string()),
            state: None,
            nonce: None,
            user_id: None,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    pub fn with_oauth_params(
        mut self,
        response_type: String,
        scope: String,
        state: Option<String>,
        nonce: Option<String>,
    ) -> Self {
        self.response_type = response_type;
        self.scope = Some(scope);
        self.state = state;
        self.nonce = nonce;
        self
    }

    pub fn with_pkce(
        mut self,
        code_challenge: Option<String>,
        code_challenge_method: Option<CodeChallengeMethod>,
    ) -> Self {
        self.code_challenge = code_challenge;
        self.code_challenge_method = code_challenge_method;
        self
    }

    pub fn with_auth_info(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }
}

pub struct GenerateTokenInput {
    pub base_url: String,
    pub realm_name: String,
    pub user_id: Uuid,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email_verified: bool,
    pub client_id: String,
    pub client_uuid: Uuid,
    pub email: String,
    pub realm_id: RealmId,
    pub scope: Option<String>,
    pub access_token_lifetime: i64,
    pub refresh_token_lifetime: i64,
    pub id_token_lifetime: i64,
    pub nonce: Option<String>,
    /// When `Some`, use this JTI for the refresh token claims and skip persisting the
    /// refresh token row (the caller has already committed it via `rotate()`).
    pub refresh_jti_override: Option<Uuid>,
}

/// Request received by the application layer for a client-scope evaluation. The application
/// service authorizes it (view access to the client) and resolves the realm/client before
/// delegating to the auth service.
pub struct EvaluateClientScopesRequest {
    pub realm_name: String,
    pub client_id: Uuid,
    /// Realm-scoped issuer base URL, already root-path scoped by the HTTP layer.
    pub base_url: String,
    pub user_id: Uuid,
    pub scope: Option<String>,
}

/// Input to the client-scope evaluation ("Evaluate") preview. The realm/client are already
/// resolved (and authorized) by the caller; this only carries what the claim assembly needs.
pub struct EvaluateClientScopesInput {
    pub base_url: String,
    pub realm_id: RealmId,
    pub realm_name: String,
    pub client_uuid: Uuid,
    /// The client's string `client_id` (e.g. `"backend"`), used as the token `azp`.
    pub client_id: String,
    pub user_id: Uuid,
    /// Requested scope string (default scopes are always applied; optional scopes apply when named here).
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluatedScope {
    pub name: String,
    pub protocol: String,
    pub default_scope_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluatedMapper {
    pub name: String,
    pub mapper_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluatedRoles {
    pub realm_roles: Vec<String>,
    pub client_roles: std::collections::HashMap<String, Vec<String>>,
}

/// Result of a client-scope evaluation: the effective scopes/mappers/roles plus the decoded
/// (unsigned, non-persisted) token claims a real token would carry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateClientScopesResult {
    pub effective_scopes: Vec<EvaluatedScope>,
    pub effective_mappers: Vec<EvaluatedMapper>,
    pub effective_roles: EvaluatedRoles,
    pub access_token: serde_json::Value,
    pub id_token: Option<serde_json::Value>,
    pub userinfo: serde_json::Value,
}

pub struct IntrospectTokenInput {
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub token: String,
    pub token_type_hint: Option<String>,
}

pub struct RevokeTokenInput {
    pub realm_name: String,
    pub client_id: String,
    pub token: String,
    pub token_type_hint: Option<String>,
}

pub struct EndSessionInput {
    pub realm_name: String,
    pub expected_issuer: String,
    pub id_token_hint: Option<String>,
    pub post_logout_redirect_uri: Option<String>,
    pub state: Option<String>,
    pub client_id: Option<String>,
}

pub struct EndSessionOutput {
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UserInfoResponse {
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
}

impl UserInfoResponse {
    pub fn from_user(user: &User) -> Self {
        Self {
            sub: user.id.to_string(),
            email: user.email.clone(),
            email_verified: Some(user.email_verified),
            family_name: user.lastname.clone(),
            given_name: user.firstname.clone(),
            name: Some(format!(
                "{} {}",
                user.firstname.as_deref().unwrap_or(""),
                user.lastname.as_deref().unwrap_or("")
            )),
            preferred_username: Some(user.username.to_string()),
        }
    }
}
