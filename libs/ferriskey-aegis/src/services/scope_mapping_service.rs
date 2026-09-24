use std::sync::Arc;

use tracing::instrument;

use crate::{
    entities::{ClientScope, ClientScopeMapping},
    ports::{
        ClientScopeMappingRepository, ClientScopePolicy, ClientScopeRepository, ScopeMappingService,
    },
    value_objects::{AssignClientScopeInput, GetClientClientScopesInput, UnassignClientScopeInput},
};

use ferriskey_authz::FerriskeyPolicy;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::entities::Client;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::ensure_policy;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, UnscopedOption};
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};
use uuid::Uuid;

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
    client_repository: Arc<C>,
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
        client_repository: Arc<C>,
        client_scope_repository: Arc<CS>,
        scope_mapping_repository: Arc<CSM>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            client_scope_repository,
            scope_mapping_repository,
            policy,
        }
    }

    async fn client_in_realm(
        &self,
        realm_scope: &RealmScope,
        client_id: Uuid,
    ) -> Result<Scoped<Client>, CoreError> {
        self.client_repository
            .get_by_id(realm_scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(realm_scope)
    }

    async fn scope_in_realm(
        &self,
        realm_scope: &RealmScope,
        scope_id: Uuid,
    ) -> Result<Scoped<ClientScope>, CoreError> {
        self.client_scope_repository
            .get_by_id(realm_scope.id(), scope_id)
            .await?
            .in_realm(realm_scope)?
            .ok_or(CoreError::NotFound)
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
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            client.id = %input.client_id,
            scope.id = %input.scope_id,
        )
    )]
    async fn assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> Result<ClientScopeMapping, CoreError> {
        let realm_scope =
            RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_update_scope(&identity, realm_scope.realm())
                .await,
            "insufficient permissions",
        )?;

        let client = self.client_in_realm(&realm_scope, input.client_id).await?;
        let client_scope = self.scope_in_realm(&realm_scope, input.scope_id).await?;

        let mapping = self
            .scope_mapping_repository
            .assign_scope_to_client(
                client.get().id,
                client_scope.get().id,
                input.is_default,
                input.is_optional,
            )
            .await?;

        Ok(mapping)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            client.id = %input.client_id,
            scope.id = %input.scope_id,
        )
    )]
    async fn unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
    ) -> Result<(), CoreError> {
        let realm_scope =
            RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_update_scope(&identity, realm_scope.realm())
                .await,
            "insufficient permissions",
        )?;

        let client = self.client_in_realm(&realm_scope, input.client_id).await?;
        let client_scope = self.scope_in_realm(&realm_scope, input.scope_id).await?;

        self.scope_mapping_repository
            .remove_scope_from_client(&client, &client_scope)
            .await?;

        Ok(())
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            client.id = %input.client_id,
        )
    )]
    async fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> {
        let realm_scope =
            RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_scope(&identity, realm_scope.realm())
                .await,
            "insufficient permissions",
        )?;

        let client = self.client_in_realm(&realm_scope, input.client_id).await?;

        let mut scopes = self
            .scope_mapping_repository
            .get_default_scopes(client.get().id)
            .await?;

        let optional_scopes = self
            .scope_mapping_repository
            .get_optional_scopes(client.get().id)
            .await?;

        scopes.extend(optional_scopes);

        scopes.retain(|scope| scope.realm_id == realm_scope.id());

        Ok(scopes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::Realm;
    use ferriskey_domain::realm::ports::MockRealmRepository;
    use ferriskey_domain::realm::scope::Unscoped;
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository};
    use uuid::Uuid;

    use crate::entities::ScopeType;
    use crate::ports::{MockClientScopeMappingRepository, MockClientScopeRepository};
    use crate::services::test_support::{
        make_admin_role, make_client, make_realm, make_scope, make_user, mock_client_repository,
        mock_policy, mock_realm_repository, row_in_realm,
    };

    type TestService = ScopeMappingServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockClientScopeRepository,
        MockClientScopeMappingRepository,
    >;

    fn build_service(
        realms: Vec<Realm>,
        roles: Vec<Role>,
        clients: Vec<Client>,
        scope_repository: MockClientScopeRepository,
        mapping_repository: MockClientScopeMappingRepository,
    ) -> TestService {
        let client_repository = Arc::new(mock_client_repository(clients));

        ScopeMappingServiceImpl::new(
            Arc::new(mock_realm_repository(realms)),
            client_repository.clone(),
            Arc::new(scope_repository),
            Arc::new(mapping_repository),
            mock_policy(roles, client_repository),
        )
    }

    /// FK-005, the claim-forgery half: a tenant admin must not be able to attach
    /// one of *his* scopes — mappers included — to a client of another tenant.
    #[tokio::test]
    async fn assign_scope_to_client_rejects_a_client_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let foreign_client = make_client(Uuid::new_v4(), realm_b.id);
        let client_id = foreign_client.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id).map(Unscoped::new);
                Box::pin(async move { Ok(found) })
            });

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_assign_scope_to_client()
            .returning(move |client_id, scope_id, _, _| {
                Box::pin(async move {
                    Ok(ClientScopeMapping {
                        client_id,
                        scope_id,
                        default_scope_type: ScopeType::Default,
                    })
                })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![foreign_client],
            scope_repository,
            mapping_repository,
        );

        let result = service
            .assign_scope_to_client(
                Identity::User(user),
                AssignClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                    scope_id,
                    is_default: true,
                    is_optional: false,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// The other end of the same call: the client is local, the scope is not.
    #[tokio::test]
    async fn assign_scope_to_client_rejects_a_scope_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let foreign_scope = make_scope(realm_b.id);
        let scope_id = foreign_scope.id;
        let client = make_client(Uuid::new_v4(), realm_a.id);
        let client_id = client.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&foreign_scope, realm_id, id).map(Unscoped::new);
                Box::pin(async move { Ok(found) })
            });

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_assign_scope_to_client()
            .returning(move |client_id, scope_id, _, _| {
                Box::pin(async move {
                    Ok(ClientScopeMapping {
                        client_id,
                        scope_id,
                        default_scope_type: ScopeType::Default,
                    })
                })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![client],
            scope_repository,
            mapping_repository,
        );

        let result = service
            .assign_scope_to_client(
                Identity::User(user),
                AssignClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                    scope_id,
                    is_default: true,
                    is_optional: false,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn assign_scope_to_client_accepts_a_client_and_a_scope_of_the_path_realm() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let client = make_client(Uuid::new_v4(), realm_a.id);
        let client_id = client.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id).map(Unscoped::new);
                Box::pin(async move { Ok(found) })
            });

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_assign_scope_to_client()
            .returning(move |client_id, scope_id, _, _| {
                Box::pin(async move {
                    Ok(ClientScopeMapping {
                        client_id,
                        scope_id,
                        default_scope_type: ScopeType::Default,
                    })
                })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![client],
            scope_repository,
            mapping_repository,
        );

        let result = service
            .assign_scope_to_client(
                Identity::User(user),
                AssignClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                    scope_id,
                    is_default: true,
                    is_optional: false,
                },
            )
            .await;

        let mapping = result.expect("a local client accepts a local scope");
        assert_eq!(mapping.client_id, client_id);
        assert_eq!(mapping.scope_id, scope_id);
    }

    #[tokio::test]
    async fn unassign_scope_from_client_rejects_a_client_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let foreign_client = make_client(Uuid::new_v4(), realm_b.id);
        let client_id = foreign_client.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id).map(Unscoped::new);
                Box::pin(async move { Ok(found) })
            });

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_remove_scope_from_client()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![foreign_client],
            scope_repository,
            mapping_repository,
        );

        let result = service
            .unassign_scope_from_client(
                Identity::User(user),
                UnassignClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                    scope_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn unassign_scope_from_client_removes_a_local_mapping() {
        let realm_a = make_realm("tenant-a");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let scope = make_scope(realm_a.id);
        let scope_id = scope.id;
        let client = make_client(Uuid::new_v4(), realm_a.id);
        let client_id = client.id;

        let mut scope_repository = MockClientScopeRepository::new();
        scope_repository
            .expect_get_by_id()
            .returning(move |realm_id, id| {
                let found = row_in_realm(&scope, realm_id, id).map(Unscoped::new);
                Box::pin(async move { Ok(found) })
            });

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_remove_scope_from_client()
            .returning(move |client, client_scope| {
                let matched = client.get().id == client_id && client_scope.get().id == scope_id;
                Box::pin(async move { matched.then_some(()).ok_or(CoreError::NotFound) })
            });

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![client],
            scope_repository,
            mapping_repository,
        );

        let result = service
            .unassign_scope_from_client(
                Identity::User(user),
                UnassignClientScopeInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                    scope_id,
                },
            )
            .await;

        result.expect("a local mapping is removable");
    }

    #[tokio::test]
    async fn get_client_scopes_rejects_a_client_from_another_realm() {
        let realm_a = make_realm("tenant-a");
        let realm_b = make_realm("tenant-b");
        let user = make_user(&realm_a);
        let role = make_admin_role(realm_a.id);
        let foreign_client = make_client(Uuid::new_v4(), realm_b.id);
        let client_id = foreign_client.id;
        let foreign_scope = make_scope(realm_b.id);

        let mut mapping_repository = MockClientScopeMappingRepository::new();
        mapping_repository
            .expect_get_default_scopes()
            .returning(move |_| {
                let scope = foreign_scope.clone();
                Box::pin(async move { Ok(vec![scope]) })
            });
        mapping_repository
            .expect_get_optional_scopes()
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let service = build_service(
            vec![realm_a],
            vec![role],
            vec![foreign_client],
            MockClientScopeRepository::new(),
            mapping_repository,
        );

        let result = service
            .get_client_scopes(
                Identity::User(user),
                GetClientClientScopesInput {
                    realm_name: "tenant-a".to_string(),
                    client_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
