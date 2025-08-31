use crate::domain::client::entities::redirect_uri::{RedirectUri, RedirectUriError};
use crate::domain::client::entities::{Client, ClientError};
use crate::domain::client::ports::{ClientRepository, RedirectUriRepository};
use crate::domain::client::value_objects::{CreateClientRequest, UpdateClientRequest};
use crate::infrastructure::client::repositories::client_postgres_repository::PostgresClientRepository;
use crate::infrastructure::client::repositories::redirect_uri_postgres_repository::PostgresRedirectUriRepository;
use uuid::Uuid;

pub mod client_postgres_repository;
pub mod redirect_uri_postgres_repository;

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

#[derive(Clone)]
pub enum RedirectUriRepoAny {
    Postgres(PostgresRedirectUriRepository),
}

impl RedirectUriRepository for RedirectUriRepoAny {
    async fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        match self {
            RedirectUriRepoAny::Postgres(repo) => {
                repo.create_redirect_uri(client_id, value, enabled).await
            }
        }
    }

    async fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        match self {
            RedirectUriRepoAny::Postgres(repo) => repo.get_by_client_id(client_id).await,
        }
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        match self {
            RedirectUriRepoAny::Postgres(repo) => repo.get_enabled_by_client_id(client_id).await,
        }
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        match self {
            RedirectUriRepoAny::Postgres(repo) => repo.update_enabled(id, enabled).await,
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        match self {
            RedirectUriRepoAny::Postgres(repo) => repo.delete(id).await,
        }
    }
}
