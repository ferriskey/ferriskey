use std::sync::Arc;

use tracing::instrument;

use crate::{
    entities::ClientScope,
    ports::{
        ClientScopePolicy, ClientScopeRepository, ClientScopeService, ProtocolMapperRepository,
    },
    value_objects::{
        CreateClientScopeInput, CreateClientScopeRequest, DeleteClientScopeInput,
        GetClientScopeInput, GetClientScopesInput, UpdateClientScopeInput,
    },
};

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

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
            .get_by_id(realm.id, input.scope_id)
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
            .update_by_id(realm.id, input.scope_id, input.payload)
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
            .delete_by_id(realm.id, input.scope_id)
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

    use crate::ports::{MockClientScopeRepository, MockProtocolMapperRepository};
    use crate::services::test_support::{
        make_admin_role, make_realm, make_scope, make_user, mock_client_repository, mock_policy,
        mock_realm_repository, row_in_realm,
    };
    use crate::value_objects::UpdateClientScopeRequest;

    type TestService = ClientScopeServiceImpl<
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
        ClientScopeServiceImpl::new(
            Arc::new(mock_realm_repository(realms)),
            Arc::new(scope_repository),
            Arc::new(mapper_repository),
            mock_policy(roles, Arc::new(mock_client_repository(vec![]))),
        )
    }

    /// FK-005: the scope lives in `tenant-b`, the request travels through
    /// `tenant-a` — reading it must be indistinguishable from a missing scope.
    #[tokio::test]
    async fn get_client_scope_rejects_a_scope_from_another_realm() {
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
        mapper_repository
            .expect_get_by_scope_id()
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .get_client_scope(
                Identity::User(user),
                GetClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn get_client_scope_returns_a_scope_of_the_path_realm() {
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
        mapper_repository
            .expect_get_by_scope_id()
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            mapper_repository,
        );

        let result = service
            .get_client_scope(
                Identity::User(user),
                GetClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                },
            )
            .await;

        assert_eq!(
            result.expect("scope of the path realm is readable").id,
            scope_id
        );
    }

    /// FK-005: writing to a scope of another tenant must not be possible.
    #[tokio::test]
    async fn update_client_scope_rejects_a_scope_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let foreign_scope = make_scope(realm_b.id);
        let scope_id = foreign_scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_update_by_id()
            .returning(move |realm_id, id, _| {
                let found = row_in_realm(&foreign_scope, realm_id, id);
                Box::pin(async move { found.ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            MockProtocolMapperRepository::new(),
        );

        let result = service
            .update_client_scope(
                Identity::User(user),
                UpdateClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    payload: UpdateClientScopeRequest {
                        name: Some("hijacked".to_string()),
                        description: None,
                        protocol: None,
                        is_default: None,
                    },
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// Mirror of the test above, and the one that pins *which* realm travels to
    /// the repository: it fails if the service forwards anything but the realm
    /// of the path.
    #[tokio::test]
    async fn update_client_scope_updates_a_scope_of_the_path_realm() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_update_by_id()
            .returning(move |realm_id, id, _| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { found.ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            MockProtocolMapperRepository::new(),
        );

        let result = service
            .update_client_scope(
                Identity::User(user),
                UpdateClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                    payload: UpdateClientScopeRequest {
                        name: Some("renamed".to_string()),
                        description: None,
                        protocol: None,
                        is_default: None,
                    },
                },
            )
            .await;

        assert_eq!(
            result.expect("a scope of the path realm is updatable").id,
            scope_id
        );
    }

    /// FK-005: deleting a scope of another tenant must not be possible.
    #[tokio::test]
    async fn delete_client_scope_rejects_a_scope_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let foreign_scope = make_scope(realm_b.id);
        let scope_id = foreign_scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_delete_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&foreign_scope, realm_id, id);
                Box::pin(async move { found.map(|_| ()).ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            MockProtocolMapperRepository::new(),
        );

        let result = service
            .delete_client_scope(
                Identity::User(user),
                DeleteClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// Same pinning as for the update: a wrong realm forwarded to the delete
    /// would make this fail.
    #[tokio::test]
    async fn delete_client_scope_deletes_a_scope_of_the_path_realm() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_delete_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id);
                Box::pin(async move { found.map(|_| ()).ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            scope_repository,
            MockProtocolMapperRepository::new(),
        );

        let result = service
            .delete_client_scope(
                Identity::User(user),
                DeleteClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    scope_id,
                },
            )
            .await;

        result.expect("a scope of the path realm is deletable");
    }
}
