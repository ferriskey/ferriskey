use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{
            AuthInput, AuthOutput, AuthSession, AuthenticateInput, AuthenticateOutput,
            AuthenticationError, AuthorizeRequestInput, AuthorizeRequestOutput,
            CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken,
        },
        value_objects::{AuthenticationResult, CreateAuthSessionRequest, GrantTypeParams},
    },
    common::entities::app_errors::CoreError,
    jwt::entities::JwkKey,
};

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeService: Clone + Send + Sync + 'static {
    fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

pub trait AuthSessionService: Clone + Send + Sync + 'static {
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

pub trait AuthSessionRepository: Clone + Send + Sync + 'static {
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
}

pub trait AuthService: Clone + Send + Sync + 'static {
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
}

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeStrategy: Clone + Send + Sync + 'static {
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

pub trait AuthenticatePort: Clone + Send + Sync + 'static {
    fn handle_token_refresh(
        &self,
        token: String,
        realm_id: Uuid,
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

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;
    mock! {
        pub AuthService {}
        impl Clone for AuthService {
            fn clone(&self) -> Self;
        }
        impl AuthService for AuthService {
            fn auth(&self, input: AuthInput) -> impl Future<Output = Result<AuthOutput, CoreError>> + Send;
            fn get_certs(&self, realm_name: String) -> impl Future<Output = Result<Vec<JwkKey>, CoreError>> + Send;
            fn exchange_token(&self, input: ExchangeTokenInput) -> impl Future<Output = Result<JwtToken, CoreError>> + Send;
            fn authorize_request(&self, input: AuthorizeRequestInput) -> impl Future<Output = Result<AuthorizeRequestOutput, CoreError>> + Send;
            fn authenticate(&self, input: AuthenticateInput) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
        }
    }
    pub fn get_mock_auth_service_with_clone_expectations() -> MockAuthService {
        let mut mock = MockAuthService::new();
        mock.expect_clone()
            .returning(|| get_mock_auth_service_with_clone_expectations());
        mock
    }
    mock! {
        pub AuthSessionService {}
        impl Clone for AuthSessionService {
            fn clone(&self) -> Self;
        }
        impl AuthSessionService for AuthSessionService {
            fn create_session(&self, dto: CreateAuthSessionRequest) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
            fn get_by_session_code(&self, session_code: Uuid) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
            fn get_by_code(&self, code: String) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
            fn update_code(&self, session_code: Uuid, code: String, user_id: Uuid) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
        }
    }
    pub fn get_mock_auth_session_service_with_clone_expectations() -> MockAuthSessionService {
        let mut mock = MockAuthSessionService::new();
        mock.expect_clone()
            .returning(|| get_mock_auth_session_service_with_clone_expectations());
        mock
    }
    mock! {
        pub AuthSessionRepository {}
        impl Clone for AuthSessionRepository {
            fn clone(&self) -> Self;
        }
        impl AuthSessionRepository for AuthSessionRepository {
            fn create(&self, session: &AuthSession) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
            fn get_by_session_code(&self, session_code: Uuid) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
            fn get_by_code(&self, code: String) -> impl Future<Output = Result<Option<AuthSession>, AuthenticationError>> + Send;
            fn update_code_and_user_id(&self, session_code: Uuid, code: String, user_id: Uuid) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
        }
    }
    pub fn get_mock_auth_session_repository_with_clone_expectations() -> MockAuthSessionRepository {
        let mut mock = MockAuthSessionRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_auth_session_repository_with_clone_expectations());
        mock
    }
    mock! {
        pub GrantTypeService {}
        impl Clone for GrantTypeService {
            fn clone(&self) -> Self;
        }
        impl GrantTypeService for GrantTypeService {
            fn authenticate_with_grant_type(&self, grant_type: GrantType, params: GrantTypeParams) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
        }
    }
    pub fn get_mock_grant_type_service_with_clone_expectations() -> MockGrantTypeService {
        let mut mock = MockGrantTypeService::new();
        mock.expect_clone()
            .returning(|| get_mock_grant_type_service_with_clone_expectations());
        mock
    }
    mock! {
        pub AuthenticatePort {}
        impl Clone for AuthenticatePort {
            fn clone(&self) -> Self;
        }
        impl AuthenticatePort for AuthenticatePort {
            fn handle_token_refresh(&self, token: String, realm_id: Uuid, auth_session: AuthSession, session_code: Uuid) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
            fn handle_user_credentials_authentication(&self, params: CredentialsAuthParams, auth_session: AuthSession) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
            fn determine_next_step(&self, auth_result: AuthenticationResult, session_code: Uuid, auth_session: AuthSession) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
            fn finalize_authentication(&self, user_id: Uuid, session_code: Uuid, auth_session: AuthSession) -> impl Future<Output = Result<AuthenticateOutput, CoreError>> + Send;
            fn build_redirect_url(&self, auth_session: &AuthSession, authorization_code: &str) -> Result<String, CoreError>;
            fn using_session_code(&self, realm_name: String, client_id: String, session_code: Uuid, username: String, password: String, base_url: String) -> impl Future<Output = Result<AuthenticationResult, CoreError>> + Send;
        }
    }
    pub fn get_mock_authenticate_port_with_clone_expectations() -> MockAuthenticatePort {
        let mut mock = MockAuthenticatePort::new();
        mock.expect_clone()
            .returning(|| get_mock_authenticate_port_with_clone_expectations());
        mock
    }
}