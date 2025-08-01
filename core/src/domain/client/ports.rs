use uuid::Uuid;

use crate::domain::client::{
    entities::{
        Client, ClientError,
        redirect_uri::{RedirectUri, RedirectUriError},
    },
    value_objects::{CreateClientRequest, CreateRedirectUriRequest, UpdateClientRequest},
};

pub trait ClientService: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        payload: CreateClientRequest,
        realm_name: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
    fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Client, ClientError>> + Send;
    fn get_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Client>, ClientError>> + Send;

    fn update_client(
        &self,
        client_id: Uuid,
        realm_name: String,
        schema: UpdateClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), ClientError>> + Send;
}

pub trait ClientRepository: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        data: CreateClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Client, ClientError>> + Send;
    fn get_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Client>, ClientError>> + Send;

    fn update_client(
        &self,
        client_id: Uuid,
        data: UpdateClientRequest,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), ClientError>> + Send;
}

pub trait RedirectUriService: Clone + Send + Sync + 'static {
    fn add_redirect_uri(
        &self,
        payload: CreateRedirectUriRequest,
        realm_name: String,
        client_id: Uuid,
    ) -> impl Future<Output = Result<RedirectUri, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, RedirectUriError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), RedirectUriError>> + Send;
}

pub trait RedirectUriRepository: Clone + Send + Sync + 'static {
    fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, RedirectUriError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, RedirectUriError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), RedirectUriError>> + Send;
}
