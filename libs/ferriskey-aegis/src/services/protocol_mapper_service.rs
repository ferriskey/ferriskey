use std::sync::Arc;

use crate::{
    entities::ProtocolMapper,
    ports::{
        ClientScopePolicy, ClientScopeRepository, ProtocolMapperRepository, ProtocolMapperService,
    },
    value_objects::{
        CreateProtocolMapperInput, CreateProtocolMapperRequest, DeleteProtocolMapperInput,
        UpdateProtocolMapperInput,
    },
};

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

#[derive(Clone, Debug)]
pub struct ProtocolMapperServiceImpl<R, U, C, UR, CS, PM>
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

impl<R, U, C, UR, CS, PM> ProtocolMapperServiceImpl<R, U, C, UR, CS, PM>
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

impl<R, U, C, UR, CS, PM> ProtocolMapperService for ProtocolMapperServiceImpl<R, U, C, UR, CS, PM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
{
    async fn create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
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

    async fn update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> Result<ProtocolMapper, CoreError> {
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

    async fn delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> Result<(), CoreError> {
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
