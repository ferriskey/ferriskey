use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::{
        entities::{
            Client, ClientError, CreateClientInput, CreateRedirectUriInput, CreateRoleInput,
            DeleteClientInput, DeleteRedirectUriInput, GetClientInput, GetClientRolesInput,
            GetClientsInput, GetRedirectUrisInput, UpdateClientInput, UpdateRedirectUriInput,
            redirect_uri::{RedirectUri, RedirectUriError},
        },
        value_objects::{CreateClientRequest, CreateRedirectUriRequest, UpdateClientRequest},
    },
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::entities::Role,
};

#[deprecated]
pub trait OldClientService: Clone + Send + Sync + 'static {
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

pub trait ClientService: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        identity: Identity,
        input: CreateClientInput,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;
    fn create_redirect_uri(
        &self,
        identity: Identity,
        input: CreateRedirectUriInput,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;
    fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn delete_client(
        &self,
        identity: Identity,
        input: DeleteClientInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn delete_redirect_uri(
        &self,
        identity: Identity,
        input: DeleteRedirectUriInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_client_roles(
        &self,
        identity: Identity,
        input: GetClientRolesInput,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
    fn get_client_by_id(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;
    fn get_clients(
        &self,
        identity: Identity,
        input: GetClientsInput,
    ) -> impl Future<Output = Result<Vec<Client>, CoreError>> + Send;

    fn get_redirect_uris(
        &self,
        identity: Identity,
        input: GetRedirectUrisInput,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;
    fn update_client(
        &self,
        identity: Identity,
        input: UpdateClientInput,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;
    fn update_redirect_uri(
        &self,
        identity: Identity,
        input: UpdateRedirectUriInput,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;
}

pub trait ClientPolicy: Clone + Send + Sync + 'static {
    fn can_create_client(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_client(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_client(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_client(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
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
