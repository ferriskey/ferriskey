use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::{
    DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration,
};

use crate::domain::authentication::value_objects::CodeChallengeMethod;
use crate::domain::common::generate_timestamp;
use crate::domain::jwt::entities::JwtClaim;
use crate::domain::realm::entities::RealmId;

// Plain OAuth2/OIDC domain types now live in the shared `ferriskey-domain` crate.
// Re-exported here so existing `crate::domain::authentication::entities::*` call sites keep
// compiling. Types that depend on `webauthn-rs` (WebAuthnChallenge, AuthSession, ...) or on
// `ferriskey-security` (AuthorizeRequestInput's JwtClaim) stay in `core` — moving them would
// pull a heavy crypto crate / an internal crate into the leaf `ferriskey-domain`.
pub use ferriskey_domain::authentication::entities::{
    AuthInput, AuthenticateInput, AuthenticateOutput, AuthenticationError, AuthenticationMethod,
    AuthenticationStepStatus, AuthorizeRequestOutput, CredentialsAuthParams, ExchangeTokenInput,
    GrantType, JwtToken, RefreshClaims, TokenIntrospectionResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebAuthnChallenge {
    Registration(PasskeyRegistration),
    Authentication(PasskeyAuthentication),
    DiscoverableAuthentication(DiscoverableAuthentication),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub code: Option<String>,
    pub authenticated: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub webauthn_challenge: Option<WebAuthnChallenge>,
    pub webauthn_challenge_issued_at: Option<DateTime<Utc>>,
    pub compass_flow_id: Option<Uuid>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

#[derive(Debug, Clone)]
pub struct AuthSessionParams {
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub code: Option<String>,
    pub authenticated: bool,
    pub webauthn_challenge: Option<WebAuthnChallenge>,
    pub webauthn_challenge_issued_at: Option<DateTime<Utc>>,
    pub compass_flow_id: Option<Uuid>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

impl AuthSession {
    pub fn new(params: AuthSessionParams) -> Self {
        let now = Utc::now();
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: params.realm_id,
            client_id: params.client_id,
            redirect_uri: params.redirect_uri,
            response_type: params.response_type,
            scope: params.scope,
            state: params.state,
            nonce: params.nonce,
            user_id: params.user_id,
            code: params.code,
            authenticated: params.authenticated,
            created_at: now,
            expires_at: now + Duration::minutes(10),
            webauthn_challenge: params.webauthn_challenge,
            webauthn_challenge_issued_at: params.webauthn_challenge_issued_at,
            compass_flow_id: params.compass_flow_id,
            code_challenge: params.code_challenge,
            code_challenge_method: params.code_challenge_method,
        }
    }
}

pub struct AuthOutput {
    pub login_url: String,
    pub session: AuthSession,
}

pub struct AuthorizeRequestInput {
    pub claims: JwtClaim,
    pub token: String,
}
