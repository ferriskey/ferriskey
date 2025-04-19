use uuid::Uuid;

use crate::{
    domain::client::{
        entities::{redirect_uri::RedirectUri, redirect_uri_error::RedirectUriError},
        ports::{
            redirect_uri_repository::RedirectUriRepository,
            redirect_uri_service::RedirectUriService,
        },
    },
    infrastructure::repositories::redirect_uri_repository::PostgresRedirectUriRepository,
};

pub type DefaultRepositery = RedirectUriServiceImpl<PostgresRedirectUriRepository>;

#[derive(Clone)]
pub struct RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    pub redirect_uri_repository: R,
    // Add any necessary fields here, such as a database connection or configuration
}
impl<R> RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    pub fn new(redirect_uri_repository: R) -> Self {
        Self {
            redirect_uri_repository,
        }
    }
}

impl<R> RedirectUriService for RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    async fn add_redirect_uri(
        &self,
        client_id: Uuid,
        uri: String,
    ) -> Result<RedirectUri, RedirectUriError> {
        self.redirect_uri_repository
            .create_redirect_uri(client_id, uri, true)
            .await
    }

    async fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        self.redirect_uri_repository
            .get_by_client_id(client_id)
            .await
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        self.redirect_uri_repository
            .get_enabled_by_client_id(client_id)
            .await
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        self.redirect_uri_repository
            .update_enabled(id, enabled)
            .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        self.redirect_uri_repository.delete(id).await
    }
}
