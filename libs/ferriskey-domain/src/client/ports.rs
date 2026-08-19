use uuid::Uuid;

use crate::auth::Identity;
use crate::client::{
    commands::{
        CreateClientInput, CreatePostLogoutRedirectUriInput, CreateRedirectUriInput,
        CreateRoleInput, DeleteClientInput, DeletePostLogoutRedirectUriInput,
        DeleteRedirectUriInput, GetClientInput, GetClientRolesInput, GetClientsInput,
        GetPostLogoutRedirectUrisInput, GetRedirectUrisInput, UpdateClientInput,
        UpdatePostLogoutRedirectUriInput, UpdateRedirectUriInput,
    },
    entities::{
        Client,
        redirect_uri::RedirectUri,
        web_origin::{WebOrigin, WebOriginValue},
    },
    value_objects::{CreateClientRequest, UpdateClientRequest},
    web_origin_resolution::ClientOriginSources,
};
use crate::common::app_errors::CoreError;
use crate::realm::{Realm, RealmId};
use crate::role::entities::Role;

pub trait ClientService: Send + Sync {
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
    fn create_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: CreatePostLogoutRedirectUriInput,
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
    fn delete_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: DeletePostLogoutRedirectUriInput,
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

    fn reveal_client_secret(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> impl Future<Output = Result<Option<String>, CoreError>> + Send;

    fn get_redirect_uris(
        &self,
        identity: Identity,
        input: GetRedirectUrisInput,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;
    fn get_post_logout_redirect_uris(
        &self,
        identity: Identity,
        input: GetPostLogoutRedirectUrisInput,
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
    fn update_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: UpdatePostLogoutRedirectUriInput,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;
}

pub trait ClientPolicy: Send + Sync {
    fn can_create_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ClientRepository: Send + Sync {
    fn create_client(
        &self,
        data: CreateClientRequest,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;

    fn get_by_id(
        &self,
        realm_id: RealmId,
        id: Uuid,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;

    fn get_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Client>, CoreError>> + Send;

    fn update_client(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        data: UpdateClientRequest,
    ) -> impl Future<Output = Result<Client, CoreError>> + Send;

    fn delete_by_id(
        &self,
        realm_id: RealmId,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RedirectUriRepository: Send + Sync {
    fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

    fn update_enabled(
        &self,
        client_id: Uuid,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

    fn delete(
        &self,
        client_id: Uuid,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait WebOriginRepository: Send + Sync {
    fn create(
        &self,
        client_id: Uuid,
        value: WebOriginValue,
    ) -> impl Future<Output = Result<WebOrigin, CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<WebOrigin>, CoreError>> + Send;

    fn delete(
        &self,
        client_id: Uuid,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Every client of `realm_name` that could contribute an origin, with the
    /// redirect URIs needed to expand [`WebOriginValue::DerivedFromRedirectUris`].
    ///
    /// Implementations must supply only *enabled* redirect URIs, and only clients
    /// that are themselves enabled: whatever comes back here is granted CORS.
    ///
    /// This sits on the CORS path, so callers are required to cache it per realm
    /// rather than call it per request.
    fn get_origin_sources_by_realm_name(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<ClientOriginSources>, CoreError>> + Send;
}
