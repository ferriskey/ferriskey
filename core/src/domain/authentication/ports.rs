use uuid::Uuid;

use crate::domain::authentication::value_objects::{
    EndSessionInput, EndSessionOutput, GenerateTokensForUserInput, GetUserInfoInput, Identity,
    IntrospectTokenInput, RevokeTokenInput, UserInfoResponse,
};
use crate::domain::realm::entities::{RealmScope, Scoped, Unscoped};
use crate::domain::{
    authentication::{
        entities::{
            AuthCompletion, AuthInput, AuthOutput, AuthSession, AuthenticateInput,
            AuthenticateOutput, AuthenticationError, AuthorizeRequestInput, AuthorizeRequestOutput,
            CredentialsAuthParams, ExchangeTokenInput, JwtToken, SsoSessionBinding,
            TokenIntrospectionResponse, WebAuthnChallenge,
        },
        value_objects::{
            AuthenticationResult, CreateAuthSessionRequest, GrantTypeParams, RegisterUserInput,
            RegisterUserOutput, RegisterUserUrlContext,
        },
    },
    common::entities::app_errors::CoreError,
    jwt::entities::JwkKey,
    user::entities::User,
};

// `GrantTypeService` now lives in `ferriskey-domain` (its signature touches only plain types).
// Re-exported so existing `crate::domain::authentication::ports::GrantTypeService` sites resolve.
pub use ferriskey_domain::authentication::ports::GrantTypeService;

pub trait AuthSessionService: Send + Sync {
    fn create_session(
        &self,
        dto: CreateAuthSessionRequest,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait AuthSessionRepository: Send + Sync {
    fn create(
        &self,
        session: &AuthSession,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<Unscoped<AuthSession>, AuthenticationError>> + Send;
    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<Option<Unscoped<AuthSession>>, AuthenticationError>> + Send;
    fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn save_webauthn_challenge(
        &self,
        session_code: Uuid,
        challenge: WebAuthnChallenge,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn take_webauthn_challenge(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<Option<WebAuthnChallenge>, AuthenticationError>> + Send;

    fn update_user_id(
        &self,
        session_code: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn update_code(
        &self,
        session_code: Uuid,
        code: String,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn update_compass_flow_id(
        &self,
        session_code: Uuid,
        compass_flow_id: Uuid,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;

    fn update_authenticated(
        &self,
        auth_session: &Scoped<AuthSession>,
        authenticated: bool,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;

    fn bind_user_session(
        &self,
        session_code: Uuid,
        user_session_id: Uuid,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;

    /// Record the "remember me" choice made at the first step of a login.
    fn set_remember_me(
        &self,
        session_code: Uuid,
        remember_me: bool,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;
}

pub trait AuthService: Send + Sync {
    fn auth(&self, input: AuthInput) -> impl Future<Output = Result<AuthOutput, CoreError>> + Send;
    fn get_certs(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<JwkKey>, CoreError>> + Send;
    fn exchange_token(
        &self,
        input: ExchangeTokenInput,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
    fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> impl Future<Output = Result<AuthorizeRequestOutput, CoreError>> + Send;
    fn authorize_login_action_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> impl Future<Output = Result<AuthorizeRequestOutput, CoreError>> + Send;
    fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn register_user(
        &self,
        url_context: RegisterUserUrlContext,
        input: RegisterUserInput,
    ) -> impl Future<Output = Result<RegisterUserOutput, CoreError>> + Send;
    fn get_userinfo(
        &self,
        identity: Identity,
        input: GetUserInfoInput,
    ) -> impl Future<Output = Result<UserInfoResponse, CoreError>> + Send;

    fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> impl Future<Output = Result<TokenIntrospectionResponse, CoreError>> + Send;
    fn revoke_token(
        &self,
        input: RevokeTokenInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn end_session(
        &self,
        input: EndSessionInput,
    ) -> impl Future<Output = Result<EndSessionOutput, CoreError>> + Send;

    fn generate_tokens_for_user(
        &self,
        input: GenerateTokensForUserInput,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;

    /// The user behind a `FERRISKEY_SSO` cookie in `realm_name`, for the
    /// browser endpoints that act as the signed-in user outside a login flow.
    fn resolve_sso_user(
        &self,
        realm_name: String,
        cookie: String,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;
}

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeStrategy: Send + Sync {
    fn authorization_code(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
    fn client_credential(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
    fn refresh_token(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
    fn password(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
}

pub trait AuthenticatePort: Send + Sync {
    fn handle_token_refresh(
        &self,
        token: String,
        scope: RealmScope,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
        scope: &RealmScope,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
        sso_session: SsoSessionBinding,
        scope: &RealmScope,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;

    fn build_auth_completion(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<AuthCompletion, CoreError>;

    fn using_session_code(
        &self,
        scope: &RealmScope,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> impl Future<Output = Result<AuthenticationResult, CoreError>> + Send;
}

pub struct LoginActionToken {
    pub jti: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub auth_session_id: Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub consumed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub trait LoginActionTokenRepository: Send + Sync {
    fn create(
        &self,
        token: LoginActionToken,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send;

    fn get_by_jti(
        &self,
        jti: Uuid,
    ) -> impl Future<Output = Result<Option<LoginActionToken>, AuthenticationError>> + Send;

    fn consume(&self, jti: Uuid) -> impl Future<Output = Result<bool, AuthenticationError>> + Send;
}

/// An SSO session just opened for a login, with the secret for the browser's
/// `FERRISKEY_SSO` cookie. Only its hash is stored.
#[derive(Clone, PartialEq, Eq)]
pub struct OpenedSsoSession {
    pub session_id: Uuid,
    pub cookie: String,
    /// `None` for a cookie that dies with the browser.
    pub max_age_secs: Option<i64>,
}

/// The cookie is a bearer secret: keep it out of logs.
impl std::fmt::Debug for OpenedSsoSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenedSsoSession")
            .field("session_id", &self.session_id)
            .field("cookie", &"<redacted>")
            .field("max_age_secs", &self.max_age_secs)
            .finish()
    }
}

/// Opens the SSO session a login completes into, for the login steps that do
/// not go through the authentication service (the MFA terminal steps).
#[cfg_attr(test, mockall::automock)]
pub trait SsoSessionPort: Send + Sync {
    /// Open a session for `user_id`, with the lifetime `client_id` grants,
    /// persistent when the user asked to be remembered and the realm allows it.
    fn open_for_login(
        &self,
        scope: &RealmScope,
        user_id: Uuid,
        client_id: Uuid,
        remember_me: bool,
    ) -> impl Future<Output = Result<OpenedSsoSession, CoreError>> + Send;
}
