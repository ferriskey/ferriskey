use crate::domain::client::entities::{Client, ClientError};
use crate::domain::client::ports::ClientRepository;
use crate::domain::client::value_objects::{CreateClientRequest, UpdateClientRequest};
use crate::infrastructure::repositories::client_repository::PostgresClientRepository;
use uuid::Uuid;

mod mappers;
pub mod repositories;

#[derive(Clone)]
pub enum ClientRepoAny {
    Postgres(PostgresClientRepository),
}

impl ClientRepository for ClientRepoAny {
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, ClientError> {
        match self {
            Self::Postgres(repo) => repo.create_client(data).await,
        }
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> Result<Client, ClientError> {
        match self {
            Self::Postgres(repo) => repo.get_by_client_id(client_id, realm_id).await,
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Client, ClientError> {
        match self {
            Self::Postgres(repo) => repo.get_by_id(id).await,
        }
    }

    async fn get_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Client>, ClientError> {
        match self {
            Self::Postgres(repo) => repo.get_by_realm_id(realm_id).await,
        }
    }

    async fn update_client(
        &self,
        client_id: Uuid,
        data: UpdateClientRequest,
    ) -> Result<Client, ClientError> {
        match self {
            Self::Postgres(repo) => repo.update_client(client_id, data).await,
        }
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), ClientError> {
        match self {
            Self::Postgres(repo) => repo.delete_by_id(id).await,
        }
    }
}
