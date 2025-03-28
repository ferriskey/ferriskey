use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::authentication::{
    entities::{error::AuthenticationError, model::JwtToken},
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
    async fn using_code(
        &self,
        client_id: String,
        code: String,
    ) -> Result<JwtToken, AuthenticationError> {
        Ok(JwtToken::new(
            "SlAV32hkKG".to_string(),
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        ))
    }

    async fn using_password(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        Ok(JwtToken::new(
            "SlAV32hkKG".to_string(),
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        ))
    }

    async fn using_credentials(
        &self,
        username: String,
        password: String,
    ) -> Result<JwtToken, AuthenticationError> {
        Ok(JwtToken::new(
            "SlAV32hkKG".to_string(),
            "Bearer".to_string(),
            "8xLOxBtZp8".to_string(),
            3600,
            "id_token".to_string(),
        ))
    }
}
