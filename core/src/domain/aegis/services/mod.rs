mod client_scope_service;
mod protocol_mapper_service;
mod scope_mapping_service;

use std::sync::Arc;

use crate::domain::{
    aegis::{
        entities::{ClientScope, ClientScopeMapping, ProtocolMapper},
        ports::{
            ClientScopeMappingRepository, ClientScopeRepository, ClientScopeService,
            ProtocolMapperRepository,
        },
        value_objects::*,
    },
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{entities::app_errors::CoreError, policies::FerriskeyPolicy},
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct ClientScopeServiceImpl<R, U, C, UR, CS, PM, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_scope_repository: Arc<CS>,
    pub(crate) protocol_mapper_repository: Arc<PM>,
    pub(crate) scope_mapping_repository: Arc<CSM>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CS, PM, CSM> ClientScopeServiceImpl<R, U, C, UR, CS, PM, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_scope_repository: Arc<CS>,
        protocol_mapper_repository: Arc<PM>,
        scope_mapping_repository: Arc<CSM>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_scope_repository,
            protocol_mapper_repository,
            scope_mapping_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CS, PM, CSM> ClientScopeService
    for ClientScopeServiceImpl<R, U, C, UR, CS, PM, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.handle_create_client_scope(identity, input).await
    }

    async fn get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.handle_get_client_scope(identity, input).await
    }

    async fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        self.handle_get_client_scopes(identity, input).await
    }

    async fn update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.handle_update_client_scope(identity, input).await
    }

    async fn delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> Result<(), CoreError> {
        self.handle_delete_client_scope(identity, input).await
    }

    async fn create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        self.handle_create_protocol_mapper(identity, input).await
    }

    async fn update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        self.handle_update_protocol_mapper(identity, input).await
    }

    async fn delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> Result<(), CoreError> {
        self.handle_delete_protocol_mapper(identity, input).await
    }

    async fn assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> Result<ClientScopeMapping, CoreError> {
        self.handle_assign_scope_to_client(identity, input).await
    }

    async fn unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
    ) -> Result<(), CoreError> {
        self.handle_unassign_scope_from_client(identity, input)
            .await
    }
}
