use uuid::Uuid;

use crate::domain::authentication::entities::{
    auth_session::{AuthSession, AuthSessionError},
    dto::CreateAuthSessionDto,
};

pub trait AuthSessionService: Clone + Send + Sync + 'static {
    fn create_session(
        &self,
        dto: CreateAuthSessionDto,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;

    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;

    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;

    fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;
}

pub trait AuthSessionRepository: Clone + Send + Sync + 'static {
    fn create(
        &self,
        session: &AuthSession,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;
    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;
    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<Option<AuthSession>, AuthSessionError>> + Send;
    fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthSessionError>> + Send;
}
