use crate::{
    application::services::ApplicationService,
    domain::{
        aegis::{
            entities::{ClientScope, ClientScopeMapping, ProtocolMapper},
            ports::{ClientScopeService, ProtocolMapperService, ScopeMappingService},
            value_objects::{
                AssignClientScopeInput, CreateClientScopeInput, CreateProtocolMapperInput,
                DeleteClientScopeInput, DeleteProtocolMapperInput, GetClientClientScopesInput,
                GetClientScopeInput, GetClientScopesInput, UnassignClientScopeInput,
                UpdateClientScopeInput, UpdateProtocolMapperInput,
            },
        },
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
    },
};

impl ClientScopeService for ApplicationService {
    async fn create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.client_scope_service
            .create_client_scope(identity, input)
            .await
    }

    async fn get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.client_scope_service
            .get_client_scope(identity, input)
            .await
    }

    async fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        self.client_scope_service
            .get_client_scopes(identity, input)
            .await
    }

    async fn update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        self.client_scope_service
            .update_client_scope(identity, input)
            .await
    }

    async fn delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> Result<(), CoreError> {
        self.client_scope_service
            .delete_client_scope(identity, input)
            .await
    }
}

impl ProtocolMapperService for ApplicationService {
    async fn create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        self.protocol_mapper_service
            .create_protocol_mapper(identity, input)
            .await
    }

    async fn update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        self.protocol_mapper_service
            .update_protocol_mapper(identity, input)
            .await
    }

    async fn delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> Result<(), CoreError> {
        self.protocol_mapper_service
            .delete_protocol_mapper(identity, input)
            .await
    }
}

impl ScopeMappingService for ApplicationService {
    async fn assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> Result<ClientScopeMapping, CoreError> {
        self.scope_mapping_service
            .assign_scope_to_client(identity, input)
            .await
    }

    async fn unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
    ) -> Result<(), CoreError> {
        self.scope_mapping_service
            .unassign_scope_from_client(identity, input)
            .await
    }

    async fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        self.scope_mapping_service
            .get_client_scopes(identity, input)
            .await
    }
}
