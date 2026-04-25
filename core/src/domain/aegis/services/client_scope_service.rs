use std::sync::Arc;

use tracing::instrument;

use crate::domain::{
    aegis::{
        entities::ClientScope,
        ports::{
            ClientScopePolicy, ClientScopeRepository, ClientScopeService, ProtocolMapperRepository,
        },
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
pub struct ClientScopeServiceImpl<R, U, C, UR, CS, PM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
{
    realm_repository: Arc<R>,
    client_scope_repository: Arc<CS>,
    protocol_mapper_repository: Arc<PM>,
    policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CS, PM> ClientScopeServiceImpl<R, U, C, UR, CS, PM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_scope_repository: Arc<CS>,
        protocol_mapper_repository: Arc<PM>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_scope_repository,
            protocol_mapper_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CS, PM> ClientScopeService for ClientScopeServiceImpl<R, U, C, UR, CS, PM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
{
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
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

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            scope.id = %input.scope_id,
        )
    )]
    async fn get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let mut client_scope = self
            .client_scope_repository
            .get_by_id(input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mappers = self
            .protocol_mapper_repository
            .get_by_scope_id(client_scope.id)
            .await?;
        client_scope.protocol_mappers = Some(mappers);

        Ok(client_scope)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_scope(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let mut client_scopes = self
            .client_scope_repository
            .find_by_realm_id(realm.id)
            .await?;

        for scope in &mut client_scopes {
            let mappers = self
                .protocol_mapper_repository
                .get_by_scope_id(scope.id)
                .await?;
            scope.protocol_mappers = Some(mappers);
        }

        Ok(client_scopes)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            scope.id = %input.scope_id,
        )
    )]
    async fn update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> Result<ClientScope, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
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

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            scope.id = %input.scope_id,
        )
    )]
    async fn delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
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
