use super::entities::error::AuthenticationError;
use super::entities::model::{GrantType, JwtToken};
use super::ports::{AuthenticationRepository, AuthenticationService};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct AuthenticationServiceImpl<A: Clone + Send + Sync + 'static>
where
    A: AuthenticationRepository,
{
    pub authentication_repository: A,
}

impl<A> AuthenticationServiceImpl<A>
where
    A: AuthenticationRepository,
{
    pub fn new(authentication_repository: A) -> Self {
        Self {
            authentication_repository,
        }
    }
}

#[async_trait]
impl<A> AuthenticationService for AuthenticationServiceImpl<A>
where
    A: AuthenticationRepository,
{
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError> {
        self.authentication_repository
            .using_code(client_id, code)
            .await
    }

    async fn using_password(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        self.authentication_repository
            .using_password(username, password)
            .await
    }

    async fn using_credentials(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        self.authentication_repository
            .using_credentials(username, password)
            .await
    }

    async fn authentificate(
        &self,
        grant_type: GrantType,
        client_id: String,
        code: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<JwtToken, AuthenticationError> {
        match grant_type {
            GrantType::Code => self.using_code(client_id, code.unwrap()).await,
            GrantType::Password => {
                self.using_password(username.unwrap(), password.unwrap())
                    .await
            }
            GrantType::Credentials => {
                self.using_credentials(username.unwrap(), password.unwrap())
                    .await
            }
        }
    }
}
