use uuid::Uuid;

use crate::domain::{
    client::{
        entities::{
            ClientError,
            redirect_uri::{RedirectUri, RedirectUriError},
        },
        ports::{ClientRepository, RedirectUriRepository, RedirectUriService},
        value_objects::CreateRedirectUriRequest,
    },
    realm::ports::RealmRepository,
};

#[derive(Debug, Clone)]
pub struct RedirectUriServiceImpl<R, RU, C>
where
    R: RealmRepository,
    RU: RedirectUriRepository,
    C: ClientRepository,
{
    pub realm_repository: R,
    pub redirect_uri_repository: RU,
    pub client_repository: C,
}

impl<R, RU, C> RedirectUriServiceImpl<R, RU, C>
where
    R: RealmRepository,
    RU: RedirectUriRepository,
    C: ClientRepository,
{
    pub fn new(realm_repository: R, redirect_uri_repository: RU, client_repository: C) -> Self {
        Self {
            realm_repository,
            redirect_uri_repository,
            client_repository,
        }
    }
}

impl<R, RU, C> RedirectUriService for RedirectUriServiceImpl<R, RU, C>
where
    R: RealmRepository,
    RU: RedirectUriRepository,
    C: ClientRepository,
{
    async fn add_redirect_uri(
        &self,
        schema: CreateRedirectUriRequest,
        realm_name: String,
        client_id: Uuid,
    ) -> Result<RedirectUri, ClientError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .ok_or(ClientError::InternalServerError)?;

        let client = self
            .client_repository
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
            .map_err(|_| RedirectUriError::InternalServerError)
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        self.redirect_uri_repository
            .get_enabled_by_client_id(client_id)
            .await
            .map_err(|_| RedirectUriError::InternalServerError)
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        self.redirect_uri_repository
            .update_enabled(id, enabled)
            .await
            .map_err(|_| RedirectUriError::InternalServerError)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        self.redirect_uri_repository.delete(id).await
    }
}
