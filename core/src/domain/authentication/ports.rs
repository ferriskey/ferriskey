use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthenticationError, GrantType, JwtToken},
    value_objects::{
        AuthenticateRequest, AuthenticationResult, CreateAuthSessionRequest, GrantTypeParams,
    },
};

pub trait AuthenticationService: Clone + Send + Sync + 'static {
    fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> impl Future<Output = Result<AuthenticationResult, AuthenticationError>> + Send;
    fn authenticate(
        &self,
        data: AuthenticateRequest,
        base_url: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

pub trait GrantTypeStrategy: Clone + Send + Sync + 'static {
    fn execute(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

pub trait GrantTypeService: Clone + Send + Sync + 'static {
    fn authenticate(
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
