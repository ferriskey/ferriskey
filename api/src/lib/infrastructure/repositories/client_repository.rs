use sqlx::PgPool;

use crate::domain::client::{
    entities::{error::ClientError, model::Client},
    ports::client_repository::ClientRepository,
};

#[derive(Debug, Clone)]
pub struct PostgresClientRepository {
    pub pool: PgPool,
}

impl PostgresClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ClientRepository for PostgresClientRepository {
    async fn create_client(
        &self,
        realm_id: uuid::Uuid,
        name: String,
        client_id: String,
        secret: Option<String>,
        enabled: bool,
        protocol: String,
        public_client: bool,
        service_account_enabled: bool,
        client_type: String,
    ) -> Result<Client, ClientError> {
        let client = Client::new(
            realm_id,
            name,
            client_id,
            secret,
            enabled,
            protocol,
            public_client,
            service_account_enabled,
            client_type,
        );

        sqlx::query!(
          r#"
          INSERT INTO clients (id, realm_id, name, client_id, secret, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
          "#,
          client.id,
          client.realm_id,
          client.name,
          client.client_id,
          client.secret,
          client.enabled,
          client.protocol,
          client.public_client,
          client.service_account_enabled,
          client.client_type,
          client.created_at,
          client.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(|_| ClientError::InternalServerError)?;

        Ok(client)
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: uuid::Uuid,
    ) -> Result<Client, ClientError> {
        let client = sqlx::query_as!(
            Client,
            r#"
            SELECT * FROM clients WHERE client_id = $1 AND realm_id = $2
            "#,
            client_id,
            realm_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| ClientError::InternalServerError)?;

        Ok(client)
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Client, ClientError> {
        let client = sqlx::query_as!(
            Client,
            r#"
            SELECT * FROM clients WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| ClientError::InternalServerError)?;
        Ok(client)
    }
}
