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
            .get_by_id(realm.id, input.scope_id)
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
            .get_by_id(realm.id, input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mapper = self
            .protocol_mapper_repository
            .update_by_id(input.scope_id, input.mapper_id, input.payload)
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
            .get_by_id(realm.id, input.scope_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.protocol_mapper_repository
            .delete_by_id(input.scope_id, input.mapper_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::Realm;
    use ferriskey_domain::realm::ports::MockRealmRepository;
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository};
    use uuid::Uuid;

    use crate::ports::{MockClientScopeRepository, MockProtocolMapperRepository};
    use crate::services::test_support::{
        make_admin_role, make_mapper, make_realm, make_scope, make_user, mock_client_repository,
        mock_policy, mock_realm_repository, row_in_realm, row_in_scope,
    };
    use crate::value_objects::UpdateProtocolMapperRequest;

    type TestService = ProtocolMapperServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockClientScopeRepository,
        MockProtocolMapperRepository,
    >;

    fn build_service(
        realms: Vec<Realm>,
        roles: Vec<Role>,
        scope_repository: MockClientScopeRepository,
        mapper_repository: MockProtocolMapperRepository,
    ) -> TestService {
        ProtocolMapperServiceImpl::new(
            Arc::new(mock_realm_repository(realms)),
            Arc::new(scope_repository),
            Arc::new(mapper_repository),
            mock_policy(roles, Arc::new(mock_client_repository(vec![]))),
        )
    }

    fn mapper_config() -> serde_json::Value {
        serde_json::json!({ "claim.name": "realm_access.roles" })
    }

    /// FK-005: a mapper may not be grafted onto a scope of another tenant.
    #[tokio::test]
    async fn create_protocol_mapper_rejects_a_scope_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let foreign_scope = make_scope(realm_b.id);
        let scope_id = foreign_scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&foreign_scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository.expect_create().returning(move |payload| {
            let mapper = make_mapper(payload.client_scope_id);
            Box::pin(async move { Ok(mapper) })
        });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .create_protocol_mapper(
                Identity::User(user),
                CreateProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    name: "forged-roles".to_string(),
                    mapper_type: "oidc-hardcoded-claim-mapper".to_string(),
                    config: mapper_config(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// The mapper belongs to another scope than the one in the path: refuse it,
    /// otherwise every mapper of the instance stays writable through any scope.
    #[tokio::test]
    async fn update_protocol_mapper_rejects_a_mapper_of_another_scope() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let foreign_mapper = make_mapper(Uuid::new_v4());
        let mapper_id = foreign_mapper.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository
            .expect_update_by_id()
            .returning(move |client_scope_id, id, _| {
                let found = row_in_scope(&foreign_mapper, client_scope_id, id);
                Box::pin(async move { found.ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .update_protocol_mapper(
                Identity::User(user),
                UpdateProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    mapper_id,
                    payload: UpdateProtocolMapperRequest {
                        name: Some("hijacked".to_string()),
                        mapper_type: None,
                        config: None,
                    },
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// Pins *which* parent id travels to the repository: forwarding anything but
    /// the scope of the path makes this fail.
    #[tokio::test]
    async fn update_protocol_mapper_updates_a_mapper_of_the_path_scope() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let mapper = make_mapper(scope_id);
        let mapper_id = mapper.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository
            .expect_update_by_id()
            .returning(move |client_scope_id, id, _| {
                let found = row_in_scope(&mapper, client_scope_id, id);
                Box::pin(async move { found.ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .update_protocol_mapper(
                Identity::User(user),
                UpdateProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    mapper_id,
                    payload: UpdateProtocolMapperRequest {
                        name: Some("renamed".to_string()),
                        mapper_type: None,
                        config: None,
                    },
                },
            )
            .await;

        assert_eq!(
            result.expect("a mapper of the path scope is updatable").id,
            mapper_id
        );
    }

    #[tokio::test]
    async fn delete_protocol_mapper_rejects_a_mapper_of_another_scope() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let foreign_mapper = make_mapper(Uuid::new_v4());
        let mapper_id = foreign_mapper.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository
            .expect_delete_by_id()
            .returning(move |client_scope_id, id| {
                let found = row_in_scope(&foreign_mapper, client_scope_id, id);
                Box::pin(async move { found.map(|_| ()).ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .delete_protocol_mapper(
                Identity::User(user),
                DeleteProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    mapper_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// Same pinning as for the update.
    #[tokio::test]
    async fn delete_protocol_mapper_deletes_a_mapper_of_the_path_scope() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let mapper = make_mapper(scope_id);
        let mapper_id = mapper.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository
            .expect_delete_by_id()
            .returning(move |client_scope_id, id| {
                let found = row_in_scope(&mapper, client_scope_id, id);
                Box::pin(async move { found.map(|_| ()).ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .delete_protocol_mapper(
                Identity::User(user),
                DeleteProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    mapper_id,
                },
            )
            .await;

        result.expect("a mapper of the path scope is deletable");
    }

    #[tokio::test]
    async fn create_protocol_mapper_accepts_a_scope_of_the_path_realm() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { Ok(found) })
            });

        let mut mapper_repository = MockProtocolMapperRepository::new();
        mapper_repository.expect_create().returning(move |payload| {
            let mapper = make_mapper(payload.client_scope_id);
            Box::pin(async move { Ok(mapper) })
        });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .create_protocol_mapper(
                Identity::User(user),
                CreateProtocolMapperInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    name: "roles".to_string(),
                    mapper_type: "oidc-usermodel-realm-role-mapper".to_string(),
                    config: mapper_config(),
                },
            )
            .await;

        assert_eq!(
            result
                .expect("a mapper can be created on a scope of the path realm")
                .client_scope_id,
            scope_id
        );
    }
}
