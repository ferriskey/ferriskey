use std::sync::Arc;

use crate::domain::{
    aegis::{
        entities::ClientScope,
        ports::{ClientScopePolicy, ClientScopeRepository, ClientScopeService},
        value_objects::{
            CreateClientScopeInput, CreateClientScopeRequest, DeleteClientScopeInput,
            GetClientScopeInput, GetClientScopesInput, UpdateClientScopeInput,
        },
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
pub struct ClientScopeServiceImpl<R, U, C, UR, CS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
{
    realm_repository: Arc<R>,
    client_scope_repository: Arc<CS>,
    policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CS> ClientScopeServiceImpl<R, U, C, UR, CS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_scope_repository: Arc<CS>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_scope_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CS> ClientScopeService for ClientScopeServiceImpl<R, U, C, UR, CS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
{
    async fn create_client_scope(
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

    async fn get_client_scope(
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

    async fn get_client_scopes(
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

    async fn update_client_scope(
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

    async fn delete_client_scope(
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
