use std::collections::HashSet;

use uuid::Uuid;

use crate::auth::Identity;
use crate::client::{
    commands::{
        CreateClientInput, CreatePostLogoutRedirectUriInput, CreateRedirectUriInput,
        CreateRoleInput, CreateSamlAttributeMapperInput, CreateWebOriginInput, DeleteClientInput,
        DeletePostLogoutRedirectUriInput, DeleteRedirectUriInput, DeleteSamlAttributeMapperInput,
        DeleteWebOriginInput, GetClientInput, GetClientRolesInput, GetClientSamlConfigInput,
        GetClientsInput, GetPostLogoutRedirectUrisInput, GetRedirectUrisInput,
        GetSamlAttributeMappersInput, GetWebOriginsInput, SetClientSamlConfigInput,
        UpdateClientInput, UpdatePostLogoutRedirectUriInput, UpdateRedirectUriInput,
    },
    entities::{
        Client,
        redirect_uri::RedirectUri,
        saml::{
            ClientSamlConfig, SamlAttributeMapper, SamlAttributeMapperDefinition,
            SamlConfigSettings,
        },
        web_origin::{Origin, WebOrigin, WebOriginValue},
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
    fn create_web_origin(
        &self,
        identity: Identity,
        input: CreateWebOriginInput,
    ) -> impl Future<Output = Result<WebOrigin, CoreError>> + Send;
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
    fn delete_web_origin(
        &self,
        identity: Identity,
        input: DeleteWebOriginInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_web_origins(
        &self,
        identity: Identity,
        input: GetWebOriginsInput,
    ) -> impl Future<Output = Result<Vec<WebOrigin>, CoreError>> + Send;
    fn get_client_saml_config(
        &self,
        identity: Identity,
        input: GetClientSamlConfigInput,
    ) -> impl Future<Output = Result<ClientSamlConfig, CoreError>> + Send;
    fn set_client_saml_config(
        &self,
        identity: Identity,
        input: SetClientSamlConfigInput,
    ) -> impl Future<Output = Result<ClientSamlConfig, CoreError>> + Send;
    fn create_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: CreateSamlAttributeMapperInput,
    ) -> impl Future<Output = Result<SamlAttributeMapper, CoreError>> + Send;
    fn get_saml_attribute_mappers(
        &self,
        identity: Identity,
        input: GetSamlAttributeMappersInput,
    ) -> impl Future<Output = Result<Vec<SamlAttributeMapper>, CoreError>> + Send;
    fn delete_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: DeleteSamlAttributeMapperInput,
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

    fn get_origin_sources_by_realm_name(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<ClientOriginSources>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ClientSamlRepository: Send + Sync {
    fn get_config_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Option<ClientSamlConfig>, CoreError>> + Send;

    fn upsert_config(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        settings: SamlConfigSettings,
    ) -> impl Future<Output = Result<ClientSamlConfig, CoreError>> + Send;

    fn create_attribute_mapper(
        &self,
        client_id: Uuid,
        definition: SamlAttributeMapperDefinition,
    ) -> impl Future<Output = Result<SamlAttributeMapper, CoreError>> + Send;

    fn get_attribute_mappers_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<SamlAttributeMapper>, CoreError>> + Send;

    fn delete_attribute_mapper(
        &self,
        client_id: Uuid,
        mapper_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait WebOriginResolver: Send + Sync {
    fn resolve_realm_origins(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<HashSet<Origin>, CoreError>> + Send;
}
