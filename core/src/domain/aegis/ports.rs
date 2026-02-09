use std::future::Future;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{Realm, RealmId};

use super::entities::{ClientScope, ClientScopeAttribute, ClientScopeMapping, ProtocolMapper};
use super::value_objects::{
    AssignClientScopeInput, CreateClientScopeInput, CreateClientScopeRequest,
    CreateProtocolMapperInput, CreateProtocolMapperRequest, DeleteClientScopeInput,
    DeleteProtocolMapperInput, GetClientScopeInput, GetClientScopesInput, UnassignClientScopeInput,
    UpdateClientScopeInput, UpdateClientScopeRequest, UpdateProtocolMapperInput,
    UpdateProtocolMapperRequest,
};

#[cfg_attr(test, mockall::automock)]
pub trait ClientScopeRepository: Send + Sync {
    fn create(
        &self,
        payload: CreateClientScopeRequest,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<ClientScope>, CoreError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

    fn find_by_name(
        &self,
        name: String,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<ClientScope>, CoreError>> + Send;

    fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateClientScopeRequest,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait ClientScopeAttributeRepository: Send + Sync {
    fn set_attribute(
        &self,
        scope_id: Uuid,
        name: String,
        value: Option<String>,
    ) -> impl Future<Output = Result<ClientScopeAttribute, CoreError>> + Send;

    fn get_attributes(
        &self,
        scope_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ClientScopeAttribute>, CoreError>> + Send;

    fn remove_attribute(
        &self,
        scope_id: Uuid,
        name: String,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait ProtocolMapperRepository: Send + Sync {
    fn create(
        &self,
        payload: CreateProtocolMapperRequest,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<ProtocolMapper>, CoreError>> + Send;

    fn get_by_scope_id(
        &self,
        scope_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ProtocolMapper>, CoreError>> + Send;

    fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateProtocolMapperRequest,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait ClientScopeMappingRepository: Send + Sync {
    fn assign_scope_to_client(
        &self,
        client_id: Uuid,
        scope_id: Uuid,
        is_default: bool,
        is_optional: bool,
    ) -> impl Future<Output = Result<ClientScopeMapping, CoreError>> + Send;

    fn remove_scope_from_client(
        &self,
        client_id: Uuid,
        scope_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_client_scopes(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ClientScopeMapping>, CoreError>> + Send;

    fn get_default_scopes(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

    fn get_optional_scopes(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;
}

pub trait ClientScopePolicy: Send + Sync {
    fn can_create_scope(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_scope(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_scope(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_scope(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait ClientScopeService: Send + Sync {
    // --- Client Scopes ---
    fn create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

    fn update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    // --- Protocol Mappers ---
    fn create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    // --- Client Scope Mappings ---
    fn assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> impl Future<Output = Result<ClientScopeMapping, CoreError>> + Send;

    fn unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
