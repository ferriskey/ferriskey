use super::entities::{
    error::AuthenticationError,
    model::{GrantType, JwtToken},
};

#[async_trait::async_trait]
pub trait AuthenticationRepository: Clone + Send + Sync + 'static {
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError>;

    async fn using_password(
        &self,
        client_id: String,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError>;

    async fn using_credentials(
        &self,
        client_id: String,
        client_secret: String,
    ) -> Result<JwtToken, AuthenticationError>;
}

#[async_trait::async_trait]
pub trait AuthenticationService: Clone + Send + Sync + 'static {
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError>;

    async fn using_password(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError>;

    async fn using_credentials(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError>;

    async fn authentificate(
        &self,
        grant_type: GrantType,
        client_id: String,
        code: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<JwtToken, AuthenticationError>;
}
