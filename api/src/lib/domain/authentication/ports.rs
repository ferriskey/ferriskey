use uuid::Uuid;

use crate::domain::client::entities::model::Client;

use super::entities::{
    error::AuthenticationError,
    model::{GrantType, JwtToken},
};

pub trait AuthenticationRepository: Clone + Send + Sync + 'static {
    fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn using_password(
        &self,
        user_id: Uuid,
        username: String,
        password: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn using_credentials(
        &self,
        realm_id: Uuid,
        client_id: String,
        client_secret: String,
    ) -> impl Future<Output = Result<Client, AuthenticationError>> + Send;
}

pub trait AuthenticationService: Clone + Send + Sync + 'static {
    fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;

    fn using_password(
        &self,
        realm_id: Uuid,
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
