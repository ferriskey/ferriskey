use std::sync::Arc;

use crate::{
    domain::client::{
        entities::{error::ClientError, model::Client},
        ports::ClientRepository,
    },
    infrastructure::db::postgres::Postgres,
};

#[derive(Debug, Clone)]
pub struct PostgresClientRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresClientRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

impl ClientRepository for PostgresClientRepository {
    async fn create_client(
        &self,
        realm_id: uuid::Uuid,
        name: String,
        client_id: String,
        enabled: bool,
        protocol: String,
    ) -> Result<Client, ClientError> {
        let client = Client::new(realm_id, name, client_id, enabled, protocol);

        sqlx::query!(
          r#"
          INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, created_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
          "#,
          client.id,
          client.realm_id,
          client.name,
          client.client_id,
          client.enabled,
          client.protocol,
          client.created_at,
          client.updated_at
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| ClientError::InternalServerError)?;

        Ok(client)
    }
}
