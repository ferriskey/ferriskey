use super::entities::{error::AuthenticationError, model::ExchangeToken};
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
    async fn exchange_token(
        &self,
        grant_type: String,
        client_id: String,
        code: String,
    ) -> Result<ExchangeToken, AuthenticationError> {
        self.authentication_repository
            .exchange_token(grant_type, client_id, code)
            .await
    }
}
