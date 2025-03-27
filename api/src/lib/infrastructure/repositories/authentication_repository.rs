use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::authentication::{
    entities::{error::AuthenticationError, model::ExchangeToken},
    ports::AuthenticationRepository,
};
use crate::infrastructure::db::postgres::Postgres;

#[derive(Debug, Clone)]
pub struct AuthenticationRepositoryImpl {
    pub postgres: Arc<Postgres>,
}

impl AuthenticationRepositoryImpl {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

#[async_trait]
impl AuthenticationRepository for AuthenticationRepositoryImpl {
    async fn exchange_token(
        &self,
        grant_type: String,
        client_id: String,
        code: String,
    ) -> Result<ExchangeToken, AuthenticationError> {
        Ok(ExchangeToken::new(
            "SlAV32hkKG".to_string(),
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        ))
    }
}
