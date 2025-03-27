use super::entities::{error::AuthenticationError, model::ExchangeToken};

#[async_trait::async_trait]
pub trait AuthenticationRepository: Clone + Send + Sync + 'static {
    async fn exchange_token(
        &self,
        grant_type: String,
        client_id: String,
        code: String,
    ) -> Result<ExchangeToken, AuthenticationError>;
}

#[async_trait::async_trait]
pub trait AuthenticationService: Clone + Send + Sync + 'static {
    async fn exchange_token(
        &self,
        grant_type: String,
        client_id: String,
        code: String,
    ) -> Result<ExchangeToken, AuthenticationError>;
}
