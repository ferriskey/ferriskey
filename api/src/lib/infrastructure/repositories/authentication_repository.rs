use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, model::JwtToken},
        ports::AuthenticationRepository,
    },
    client::entities::model::Client,
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
        user_id: Uuid,
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
        realm_id: Uuid,
        client_id: String,
        client_secret: String,
    ) -> Result<Client, AuthenticationError> {
        sqlx::query_as!(
            Client,
            "SELECT * FROM clients WHERE realm_id = $1 AND client_id = $2 AND secret = $3",
            realm_id,
            client_id,
            client_secret,
        )
        .fetch_one(&*self.postgres.pool)
        .await
        .map_err(|_| AuthenticationError::Invalid)
    }
}
