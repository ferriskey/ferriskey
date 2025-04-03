use uuid::Uuid;

use super::entities::{
    error::AuthenticationError,
    model::{GrantType, JwtToken},
};

pub mod auth_session;

pub trait AuthenticationService: Clone + Send + Sync + 'static {
    fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn using_password(
        &self,
        realm_id: Uuid,
        client_id: String,
        username: String,
        password: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn using_credentials(
        &self,
        realm_id: Uuid,
        client_id: String,
        client_secret: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn authentificate(
        &self,
        realm_name: String,
        grant_type: GrantType,
        client_id: String,
        client_secret: Option<String>,
        code: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
