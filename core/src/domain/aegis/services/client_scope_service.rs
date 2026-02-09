use crate::domain::{
    aegis::{
        entities::ClientScope,
        ports::{
            ClientScopeMappingRepository, ClientScopePolicy, ClientScopeRepository,
            ProtocolMapperRepository,
        },
        value_objects::{
            CreateClientScopeInput, CreateClientScopeRequest, DeleteClientScopeInput,
            GetClientScopeInput, GetClientScopesInput, UpdateClientScopeInput,
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
    pub(super) async fn handle_create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_create_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client_scope = self
            .client_scope_repository
            .create(CreateClientScopeRequest {
                realm_id: realm.id,
                name: input.name,
                description: input.description,
                protocol: input.protocol,
                is_default: input.is_default,
            })
            .await?;

        Ok(client_scope)
    }

    pub(super) async fn handle_get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client_scope = self
            .client_scope_repository
            .get_by_id(input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        Ok(client_scope)
    }

    pub(super) async fn handle_get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client_scopes = self
            .client_scope_repository
            .find_by_realm_id(realm.id)
            .await?;

        Ok(client_scopes)
    }

    pub(super) async fn handle_update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
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

        let client_scope = self
            .client_scope_repository
            .update_by_id(input.scope_id, input.payload)
            .await?;

        Ok(client_scope)
    }

    pub(super) async fn handle_delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.client_scope_repository
            .delete_by_id(input.scope_id)
            .await?;

        Ok(())
    }
}
