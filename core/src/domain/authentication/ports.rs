use uuid::Uuid;

use crate::domain::authentication::value_objects::{
    GetUserInfoInput, Identity, IntrospectTokenInput, UserInfoResponse,
};
use crate::domain::realm::entities::RealmId;
use crate::domain::{
    authentication::{
        entities::{
            AuthInput, AuthOutput, AuthSession, AuthenticateInput, AuthenticateOutput,
            AuthenticationError, AuthorizeRequestInput, AuthorizeRequestOutput,
            CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken,
            TokenIntrospectionResponse, WebAuthnChallenge,
        },
        value_objects::{
            AuthenticationResult, CreateAuthSessionRequest, GrantTypeParams, RegisterUserInput,
        },
    },
    common::entities::app_errors::CoreError,
    jwt::entities::JwkKey,
};

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeService: Send + Sync {
    fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

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
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<Option<AuthSession>, AuthenticationError>> + Send;
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
    fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn register_user(
        &self,
        url: String,
        input: RegisterUserInput,
    ) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
    fn get_userinfo(
        &self,
        identity: Identity,
        input: GetUserInfoInput,
    ) -> impl Future<Output = Result<UserInfoResponse, CoreError>> + Send;

    fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> impl Future<Output = Result<TokenIntrospectionResponse, CoreError>> + Send;
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
        realm_id: RealmId,
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
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
    fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;

    fn build_redirect_url(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<String, CoreError>;

    fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> impl Future<Output = Result<AuthenticationResult, CoreError>> + Send;
}
