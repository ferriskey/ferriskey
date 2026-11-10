use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::{
    DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration,
};

use crate::domain::authentication::assurance::{
    AcrLoaMap, AcrRequest, FactorUse, LevelOfAssurance, VerifiedFactor, factors_of,
};
use crate::domain::authentication::value_objects::CodeChallengeMethod;
use crate::domain::common::generate_timestamp;
use crate::domain::jwt::entities::JwtClaim;
use crate::domain::realm::entities::{RealmId, RealmOwned};

// Plain OAuth2/OIDC domain types now live in the shared `ferriskey-domain` crate.
// Re-exported here so existing `crate::domain::authentication::entities::*` call sites keep
// compiling. Types that depend on `webauthn-rs` (WebAuthnChallenge, AuthSession, ...) or on
// `ferriskey-security` (AuthorizeRequestInput's JwtClaim) stay in `core` — moving them would
// pull a heavy crypto crate / an internal crate into the leaf `ferriskey-domain`.
pub use ferriskey_domain::authentication::entities::{
    AuthCompletion, AuthInput, AuthProtocol, AuthenticateInput, AuthenticateOutput,
    AuthenticationError, AuthenticationMethod, AuthenticationStepStatus, AuthorizeRequestOutput,
    CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken, RefreshClaims,
    SsoSessionBinding, TokenIntrospectionResponse,
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
    pub protocol: AuthProtocol,
    pub redirect_uri: String,
    pub response_type: Option<String>,
    pub scope: Option<String>,
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
    pub user_session_id: Option<Uuid>,
    /// The "remember me" choice made on the login page, kept until the login
    /// completes so the session it opens knows whether to persist.
    pub remember_me: bool,
    /// A live SSO session the browser still held when it was sent to the
    /// login page; a login for the same user re-authenticates into it.
    pub reauth_session_id: Option<Uuid>,
    pub consent_token_hash: Option<String>,
    pub prompt_consent: bool,
    pub issuer: Option<String>,
    pub resource: Option<String>,
    /// The `acr` values the client asked for, space-separated.
    pub acr_values: Option<String>,
    /// The factors answered so far in this login.
    pub verified_factors: Vec<VerifiedFactor>,
}

impl AuthSession {
    pub fn acr_request(&self) -> AcrRequest {
        AcrRequest::parse(self.acr_values.as_deref())
    }

    /// The factors `user_id` answered in this login.
    pub fn factors_of(&self, user_id: Uuid) -> Vec<FactorUse> {
        factors_of(&self.verified_factors, user_id)
    }

    /// The level the client asked for that the factors `user_id` answered so
    /// far do not reach yet.
    pub fn level_still_owed(&self, user_id: Uuid, map: &AcrLoaMap) -> Option<LevelOfAssurance> {
        let required = self.acr_request().required_level(map)?;
        let reached = LevelOfAssurance::of(&self.factors_of(user_id));

        reached
            .is_none_or(|reached| reached < required)
            .then_some(required)
    }
}

impl RealmOwned for AuthSession {
    fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}

#[derive(Debug, Clone)]
pub struct AuthSessionParams {
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub protocol: AuthProtocol,
    pub redirect_uri: String,
    pub response_type: Option<String>,
    pub scope: Option<String>,
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
    pub prompt_consent: bool,
    pub issuer: Option<String>,
    pub resource: Option<String>,
    pub acr_values: Option<String>,
}

impl AuthSession {
    pub fn new(params: AuthSessionParams) -> Self {
        let now = Utc::now();
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: params.realm_id,
            client_id: params.client_id,
            protocol: params.protocol,
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
            user_session_id: None,
            remember_me: false,
            reauth_session_id: None,
            consent_token_hash: None,
            prompt_consent: params.prompt_consent,
            issuer: params.issuer,
            resource: params.resource,
            acr_values: params.acr_values,
            verified_factors: Vec::new(),
        }
    }
}

pub struct AuthOutput {
    pub login_url: String,
    pub session: AuthSession,
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenUsage {
    ManagementApi,
    AnyClient,
}

pub struct AuthorizeRequestInput {
    pub claims: JwtClaim,
    pub token: String,
    pub realm_name: Option<String>,
    pub usage: TokenUsage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::assurance::AuthenticationFactor;

    fn login(acr_values: Option<&str>) -> AuthSession {
        AuthSession::new(AuthSessionParams {
            realm_id: RealmId::default(),
            client_id: Uuid::new_v4(),
            protocol: AuthProtocol::OpenIdConnect,
            redirect_uri: "https://app.example/cb".to_string(),
            response_type: Some("code".to_string()),
            scope: Some("openid".to_string()),
            state: None,
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
            prompt_consent: false,
            issuer: None,
            resource: None,
            acr_values: acr_values.map(str::to_string),
        })
    }

    #[test]
    fn a_multi_factor_request_is_owed_until_a_second_factor_is_answered() {
        let user_id = Uuid::new_v4();
        let mut session = login(Some("2"));
        let map = AcrLoaMap::default();

        assert_eq!(
            session.level_still_owed(user_id, &map),
            Some(LevelOfAssurance::MultiFactor)
        );

        session
            .verified_factors
            .push(VerifiedFactor::now(user_id, AuthenticationFactor::Password));
        assert_eq!(
            session.level_still_owed(user_id, &map),
            Some(LevelOfAssurance::MultiFactor)
        );

        session
            .verified_factors
            .push(VerifiedFactor::now(user_id, AuthenticationFactor::Otp));
        assert_eq!(session.level_still_owed(user_id, &map), None);
    }

    #[test]
    fn a_second_factor_answered_by_another_account_does_not_count() {
        let user_id = Uuid::new_v4();
        let mut session = login(Some("2"));
        session.verified_factors.push(VerifiedFactor::now(
            Uuid::new_v4(),
            AuthenticationFactor::HardwareKey,
        ));

        assert_eq!(
            session.level_still_owed(user_id, &AcrLoaMap::default()),
            Some(LevelOfAssurance::MultiFactor)
        );
    }

    #[test]
    fn nothing_is_owed_without_a_request_the_realm_knows() {
        let user_id = Uuid::new_v4();
        let map = AcrLoaMap::default();

        assert_eq!(login(None).level_still_owed(user_id, &map), None);
        assert_eq!(
            login(Some("urn:example:unknown")).level_still_owed(user_id, &map),
            None
        );
    }
}
