use crate::domain::client::entities::redirect_uri::{RedirectUri, RedirectUriError};
use crate::domain::client::ports::RedirectUriRepository;
use crate::infrastructure::repositories::redirect_uri_repository::PostgresRedirectUriRepository;
use uuid::Uuid;

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
