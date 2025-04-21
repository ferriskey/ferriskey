use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::http::client::validators::CreateRedirectUriValidator,
    domain::{
        client::{
            entities::{
                error::ClientError, redirect_uri::RedirectUri, redirect_uri_error::RedirectUriError,
            },
            ports::{
                client_service::ClientService, redirect_uri_repository::RedirectUriRepository,
                redirect_uri_service::RedirectUriService,
            },
        },
        realm::{ports::realm_service::RealmService, services::realm_service::DefaultRealmService},
    },
    infrastructure::repositories::redirect_uri_repository::PostgresRedirectUriRepository,
};

use super::client_service::DefaultClientService;

pub type DefaultRedirectUriService = RedirectUriServiceImpl<PostgresRedirectUriRepository>;

#[derive(Clone)]
pub struct RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    pub redirect_uri_repository: R,
    pub realm_service: Arc<DefaultRealmService>,
    pub client_service: Arc<DefaultClientService>,
}
impl<R> RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    pub fn new(
        redirect_uri_repository: R,
        realm_service: Arc<DefaultRealmService>,
        client_service: Arc<DefaultClientService>,
    ) -> Self {
        Self {
            redirect_uri_repository,
            realm_service,
            client_service,
        }
    }
}

impl<R> RedirectUriService for RedirectUriServiceImpl<R>
where
    R: RedirectUriRepository,
{
    async fn add_redirect_uri(
        &self,
        schema: CreateRedirectUriValidator,
        realm_name: String,
        client_id: Uuid,
    ) -> Result<RedirectUri, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let client = self
            .client_service
            .get_by_id(client_id)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        if client.realm_id != realm.id {
            return Err(ClientError::NotFound);
        }

        let redirect_uri = self
            .redirect_uri_repository
            .create_redirect_uri(client.id, schema.value, schema.enabled)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Ok(redirect_uri)
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
