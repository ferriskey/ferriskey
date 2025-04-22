use sqlx::PgPool;

use crate::domain::{
    client::{
        entities::{dto::CreateClientDto, error::ClientError, model::Client},
        ports::client_repository::ClientRepository,
    },
    utils::{generate_timestamp, generate_uuid_v7},
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
    async fn create_client(&self, data: CreateClientDto) -> Result<Client, ClientError> {
        let (now, _) = generate_timestamp();
        sqlx::query_as!(
          Client,
          r#"
          INSERT INTO clients (id, realm_id, name, client_id, secret, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *
          "#,
          generate_uuid_v7(),
          data.realm_id,
          data.name,
          data.client_id,
          data.secret,
          data.enabled,
          data.protocol,
          data.public_client,
          data.service_account_enabled,
          data.client_type,
          now,
          now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| ClientError::InternalServerError)
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
