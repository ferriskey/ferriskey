use uuid::Uuid;

use crate::domain::authentication::entities::{
    dto::AuthenticateDto, error::AuthenticationError, jwt_token::JwtToken,
};

pub trait AuthenticationService: Clone + Send + Sync + 'static {
    fn using_session_code(
        &self,
        realm_id: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
    ) -> impl Future<Output = Result<String, AuthenticationError>> + Send;

    fn authenticate(
        &self,
        data: AuthenticateDto,
        base_url: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
