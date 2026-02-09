use crate::domain::{
    aegis::{
        entities::ProtocolMapper,
        ports::{
            ClientScopeMappingRepository, ClientScopePolicy, ClientScopeRepository,
            ProtocolMapperRepository,
        },
        value_objects::{
            CreateProtocolMapperInput, CreateProtocolMapperRequest, DeleteProtocolMapperInput,
            UpdateProtocolMapperInput,
        },
    },
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{entities::app_errors::CoreError, policies::ensure_policy},
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

use super::ClientScopeServiceImpl;

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
    pub(super) async fn handle_create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.client_scope_repository
            .get_by_id(input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mapper = self
            .protocol_mapper_repository
            .create(CreateProtocolMapperRequest {
                client_scope_id: input.scope_id,
                name: input.name,
                mapper_type: input.mapper_type,
                config: input.config,
            })
            .await?;

        Ok(mapper)
    }

    pub(super) async fn handle_update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.client_scope_repository
            .get_by_id(input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mapper = self
            .protocol_mapper_repository
            .update_by_id(input.mapper_id, input.payload)
            .await?;

        Ok(mapper)
    }

    pub(super) async fn handle_delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.client_scope_repository
            .get_by_id(input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.protocol_mapper_repository
            .delete_by_id(input.mapper_id)
            .await?;

        Ok(())
    }
}
