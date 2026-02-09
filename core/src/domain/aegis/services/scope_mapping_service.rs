use crate::domain::{
    aegis::{
        entities::ClientScopeMapping,
        ports::{
            ClientScopeMappingRepository, ClientScopePolicy, ClientScopeRepository,
            ProtocolMapperRepository,
        },
        value_objects::{AssignClientScopeInput, UnassignClientScopeInput},
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
    pub(super) async fn handle_assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> Result<ClientScopeMapping, CoreError> {
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

        let mapping = self
            .scope_mapping_repository
            .assign_scope_to_client(
                input.client_id,
                input.scope_id,
                input.is_default,
                input.is_optional,
            )
            .await?;

        Ok(mapping)
    }

    pub(super) async fn handle_unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
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

        self.scope_mapping_repository
            .remove_scope_from_client(input.client_id, input.scope_id)
            .await?;

        Ok(())
    }
}
