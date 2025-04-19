use uuid::Uuid;

use crate::domain::authentication::entities::{
    error::AuthenticationError, grant_type::GrantType, jwt_token::JwtToken,
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

    fn authentificate(
        &self,
        realm_name: String,
        grant_type: GrantType,
        client_id: String,
        client_secret: Option<String>,
        code: Option<String>,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
