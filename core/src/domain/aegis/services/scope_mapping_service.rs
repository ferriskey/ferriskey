use std::sync::Arc;

use crate::domain::{
    aegis::{
        entities::ClientScopeMapping,
        ports::{
            ClientScopeMappingRepository, ClientScopePolicy, ClientScopeRepository,
            ScopeMappingService,
        },
        value_objects::{AssignClientScopeInput, UnassignClientScopeInput},
    },
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct ScopeMappingServiceImpl<R, U, C, UR, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    realm_repository: Arc<R>,
    client_scope_repository: Arc<CS>,
    scope_mapping_repository: Arc<CSM>,
    policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CS, CSM> ScopeMappingServiceImpl<R, U, C, UR, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_scope_repository: Arc<CS>,
        scope_mapping_repository: Arc<CSM>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_scope_repository,
            scope_mapping_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CS, CSM> ScopeMappingService for ScopeMappingServiceImpl<R, U, C, UR, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn assign_scope_to_client(
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

    async fn unassign_scope_from_client(
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
