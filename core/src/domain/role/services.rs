use std::sync::Arc;

use tracing::warn;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::{
        entities::{RealmScope, Scoped},
        ports::RealmRepository,
    },
    role::{
        entities::{CreateRoleInput, Role, UpdateRoleInput},
        ports::{RolePolicy, RoleRepository, RoleService},
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
    seawatch::{EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    user::ports::{UserRepository, UserRoleRepository},
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};

#[derive(Clone, Debug)]
pub struct RoleServiceImpl<R, U, C, UR, RO, SE, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    W: WebhookRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, RO, SE, W> RoleServiceImpl<R, U, C, UR, RO, SE, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    W: WebhookRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        role_repository: Arc<RO>,
        security_event_repository: Arc<SE>,
        webhook_repository: Arc<W>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            role_repository,
            security_event_repository,
            webhook_repository,
            policy,
        }
    }

    async fn load_role_in_realm(
        &self,
        role_id: Uuid,
        scope: &RealmScope,
    ) -> Result<Scoped<Role>, CoreError> {
        self.role_repository
            .get_by_id(role_id)
            .await?
            .ok_or_else(|| {
                warn!(role_id = %role_id, "Role not found");
                CoreError::NotFound
            })?
            .in_realm(scope)
    }
}

impl<R, U, C, UR, RO, SE, W> RoleService for RoleServiceImpl<R, U, C, UR, RO, SE, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    W: WebhookRepository,
{
    async fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> Result<Role, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        if !self
            .policy
            .can_grant_permissions(&identity, &realm, &input.permissions)
            .await?
        {
            return Err(CoreError::Forbidden(
                "cannot grant a permission you do not hold".to_string(),
            ));
        }

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: None,
                description: input.description,
                name: input.name,
                permissions: input.permissions,
                realm_id,
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::RoleCreated,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RoleCreated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(role)
    }

    async fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_delete_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let role = self.load_role_in_realm(role_id, &scope).await?;
        self.role_repository.delete_by_id(&role).await?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::RoleRemoved,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RoleDeleted,
                    realm_id.into(),
                    Some(role.into_inner()),
                ),
            )
            .await?;

        Ok(())
    }

    async fn get_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<Role, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.load_role_in_realm(role_id, &scope)
            .await
            .map(Scoped::into_inner)
    }

    async fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Role>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> Result<Role, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let scoped = self.load_role_in_realm(input.role_id, &scope).await?;

        let role = self
            .role_repository
            .update_by_id(
                &scoped,
                UpdateRoleRequest {
                    description: input.description,
                    name: input.name,
                    require_mfa: input.require_mfa,
                },
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::RoleUpdated,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RoleUpdated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(role)
    }

    async fn update_role_permissions(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
        permissions: Vec<String>,
    ) -> Result<Role, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        if !self
            .policy
            .can_grant_permissions(&identity, &realm, &permissions)
            .await?
        {
            return Err(CoreError::Forbidden(
                "cannot grant a permission you do not hold".to_string(),
            ));
        }

        let scoped = self.load_role_in_realm(role_id, &scope).await?;

        let role = self
            .role_repository
            .update_permissions_by_id(&scoped, UpdateRolePermissionsRequest { permissions })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::RolePermissionUpdated,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RolePermissionUpdated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(role)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use uuid::Uuid;

    use crate::domain::{
        authentication::value_objects::Identity,
        client::{entities::Client, ports::MockClientRepository},
        common::{
            entities::app_errors::CoreError,
            policies::FerriskeyPolicy,
            services::tests::{
                assert_core_erro, assert_success, create_test_realm, create_test_realm_with_name,
                create_test_role, create_test_role_with_params, create_test_user,
                create_test_user_with_realm,
            },
        },
        realm::{
            entities::{Realm, RealmId, Scoped, Unscoped},
            ports::MockRealmRepository,
        },
        role::{
            entities::{CreateRoleInput, Role, UpdateRoleInput, permission::Permissions},
            ports::{MockRoleRepository, RoleService},
            services::RoleServiceImpl,
            value_objects::CreateRoleRequest,
        },
        seawatch::ports::MockSecurityEventRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
        webhook::{entities::webhook_payload::WebhookPayload, ports::MockWebhookRepository},
    };
    use std::{sync::Arc, vec};

    struct RoleServiceTestBuilder {
        realm_repo: Arc<MockRealmRepository>,
        role_repo: Arc<MockRoleRepository>,
        security_event_repo: Arc<MockSecurityEventRepository>,
        webhook_repo: Arc<MockWebhookRepository>,
        user_role_repo: Arc<MockUserRoleRepository>,
        client_repo: Arc<MockClientRepository>,
        user_repo: Arc<MockUserRepository>,
    }

    impl RoleServiceTestBuilder {
        fn new() -> Self {
            Self {
                realm_repo: Arc::new(MockRealmRepository::new()),
                role_repo: Arc::new(MockRoleRepository::new()),
                security_event_repo: Arc::new(MockSecurityEventRepository::new()),
                webhook_repo: Arc::new(MockWebhookRepository::new()),
                user_role_repo: Arc::new(MockUserRoleRepository::new()),
                client_repo: Arc::new(MockClientRepository::new()),
                user_repo: Arc::new(MockUserRepository::new()),
            }
        }

        fn with_successful_realm_lookup(mut self, realm_name: &str, realm: Realm) -> Self {
            let name = realm_name.to_string();
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(eq(name))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(realm)) }));

            self
        }

        fn with_missing_realm_lookup(mut self, realm_name: &str) -> Self {
            let name = realm_name.to_string();
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(eq(name))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(None) }));

            self
        }

        fn with_successful_role_lookup(mut self, role_id: Uuid, role: Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_get_by_id()
                .with(eq(role_id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(Unscoped::new(role))) }));
            self
        }

        fn with_successful_role_create(mut self, expected: CreateRoleRequest, role: Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_create()
                .with(eq(expected))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(role) }));
            self
        }

        fn with_user_roles(mut self, user_id: Uuid, roles: Vec<Role>) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_get_user_roles()
                .with(eq(user_id))
                .returning(move |_| {
                    let roles = roles.clone();
                    Box::pin(async move { Ok(roles) })
                });
            self
        }

        fn with_no_user_roles(mut self) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_get_user_roles()
                .return_once(move |_| Box::pin(async move { Ok(vec![]) }));
            self
        }

        fn with_successful_client_lookup(
            mut self,
            client_id: &str,
            realm_id: RealmId,
            client: Client,
        ) -> Self {
            Arc::get_mut(&mut self.client_repo)
                .unwrap()
                .expect_get_by_client_id()
                .with(eq(client_id.to_string()), eq(realm_id))
                .times(1)
                .return_once(move |_, _| Box::pin(async move { Ok(Unscoped::new(client)) }));
            self
        }

        fn with_successful_role_update(mut self, role_id: Uuid, role: Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_update_by_id()
                .with(
                    function(move |role: &Scoped<Role>| role.get().id == role_id),
                    always(),
                )
                .times(1)
                .return_once(move |_, _| Box::pin(async move { Ok(role) }));
            self
        }

        fn with_successful_role_permissions_update(mut self, role_id: Uuid, role: Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_update_permissions_by_id()
                .with(
                    function(move |role: &Scoped<Role>| role.get().id == role_id),
                    always(),
                )
                .times(1)
                .return_once(move |_, _| Box::pin(async move { Ok(role) }));
            self
        }

        fn with_security_event_store(mut self) -> Self {
            Arc::get_mut(&mut self.security_event_repo)
                .unwrap()
                .expect_store_event()
                .times(1)
                .return_once(|_| Box::pin(async move { Ok(()) }));
            self
        }

        fn with_role_webhook_notify(mut self) -> Self {
            Arc::get_mut(&mut self.webhook_repo)
                .unwrap()
                .expect_notify::<Role>()
                .times(1)
                .return_once(|_, _: WebhookPayload<Role>| Box::pin(async move { Ok(()) }));
            self
        }

        fn build(
            self,
        ) -> RoleServiceImpl<
            MockRealmRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
            MockRoleRepository,
            MockSecurityEventRepository,
            MockWebhookRepository,
        > {
            let policy = FerriskeyPolicy::new(
                self.user_repo.clone(),
                self.client_repo.clone(),
                self.user_role_repo.clone(),
            );
            RoleServiceImpl::new(
                self.realm_repo,
                self.role_repo,
                self.security_event_repo,
                self.webhook_repo,
                Arc::new(policy),
            )
        }
    }

    #[tokio::test]
    async fn test_get_role_success() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let user_role_with_permissions = create_test_role_with_params(
            realm.id,
            "viewer-role",
            vec![Permissions::ViewRoles.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![user_role_with_permissions])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        let result = service
            .get_role(identity, realm.name.clone(), role.id)
            .await;

        // Assert
        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
        assert_eq!(returned_role.name, "test-role");
        assert_eq!(returned_role.realm_id, realm.id);
    }

    #[tokio::test]
    async fn test_create_realm_role_success() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let created_role = create_test_role_with_params(
            realm.id,
            "service-manager",
            vec![
                Permissions::ManageUsers.name(),
                Permissions::ViewRoles.name(),
            ],
            None,
        );
        let create_request = CreateRoleRequest {
            name: "service-manager".to_string(),
            description: Some("Realm-wide service management role".to_string()),
            permissions: vec![
                Permissions::ManageUsers.name(),
                Permissions::ViewRoles.name(),
            ],
            realm_id: realm.id,
            client_id: None,
        };
        let admin_role = create_test_role_with_params(
            realm.id,
            "realm-admin",
            vec![Permissions::ManageRealm.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![admin_role])
            .with_successful_role_create(create_request, created_role.clone())
            .with_security_event_store()
            .with_role_webhook_notify()
            .build();

        let result = service
            .create_role(
                identity,
                CreateRoleInput {
                    realm_name: realm.name,
                    name: "service-manager".to_string(),
                    description: Some("Realm-wide service management role".to_string()),
                    permissions: vec![
                        Permissions::ManageUsers.name(),
                        Permissions::ViewRoles.name(),
                    ],
                },
            )
            .await;

        let role = assert_success(result);
        assert_eq!(role.client_id, None);
        assert_eq!(role.name, "service-manager");
    }

    #[tokio::test]
    async fn test_create_realm_role_with_missing_realm_fails() {
        let realm = create_test_realm_with_name("missing-realm");
        let user = create_test_user(realm.id);
        let identity = Identity::User(user);

        let service = RoleServiceTestBuilder::new()
            .with_missing_realm_lookup(&realm.name)
            .build();

        let result = service
            .create_role(
                identity,
                CreateRoleInput {
                    realm_name: realm.name,
                    name: "service-manager".to_string(),
                    description: Some("Realm-wide service management role".to_string()),
                    permissions: vec![Permissions::ManageUsers.name()],
                },
            )
            .await;

        assert_core_erro(result, CoreError::InvalidRealm);
    }

    #[tokio::test]
    async fn test_get_role_success_with_manage_users_permissions() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);

        let identity = Identity::User(user.clone());

        let admin_role = create_test_role_with_params(
            realm.id,
            "admin-role",
            vec![Permissions::ManageUsers.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        let result = service.get_role(identity, realm.name, role.id).await;

        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
    }

    #[tokio::test]
    async fn test_get_role_success_with_manage_realm_permission() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let realm_admin_role = create_test_role_with_params(
            realm.id,
            "realm-admin",
            vec![Permissions::ManageRealm.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![realm_admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
    }

    #[tokio::test]
    async fn test_get_role_user_without_realm_should_fail() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user(realm.id);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_no_user_roles()
            .build();

        let result = service
            .get_role(identity, realm.name.clone(), role.id)
            .await;

        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_insufficient_permissions() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let insufficient_role = create_test_role_with_params(
            realm.id,
            "basic-user",
            vec!["some_other_permission".to_string()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![insufficient_role])
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_no_roles_at_all() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_no_user_roles()
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_cross_realm_access_from_master() {
        // Arrange
        let master_realm = create_test_realm_with_name("master");
        let target_realm = create_test_realm_with_name("target-realm");
        let user_in_master = create_test_user_with_realm(&master_realm);
        let role_in_target = create_test_role(target_realm.id);
        let identity = Identity::User(user_in_master.clone());

        let target_realm_client_id = format!("{}-realm", target_realm.name);
        let target_realm_client =
            Client::from_realm_and_client_id(master_realm.id, target_realm_client_id.clone());

        let cross_realm_role = create_test_role_with_params(
            master_realm.id,
            "cross-realm-admin",
            vec![Permissions::ViewRoles.name()],
            Some(target_realm_client.id),
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&target_realm.name, target_realm.clone())
            .with_user_roles(user_in_master.id, vec![cross_realm_role])
            .with_successful_client_lookup(
                &target_realm_client_id,
                master_realm.id,
                target_realm_client,
            )
            .with_successful_role_lookup(role_in_target.id, role_in_target.clone())
            .build();

        let result = service
            .get_role(identity, target_realm.name, role_in_target.id)
            .await;

        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role_in_target.id);
    }

    #[tokio::test]
    async fn test_get_role_scoped_to_url_realm_returns_not_found() {
        let principal_realm = create_test_realm_with_name("principal");
        let neighbor_realm = create_test_realm_with_name("neighbor");
        let admin = create_test_user_with_realm(&principal_realm);
        let identity = Identity::User(admin.clone());

        let admin_role = create_test_role_with_params(
            principal_realm.id,
            "principal-viewer",
            vec![Permissions::ViewRoles.name()],
            None,
        );

        let foreign_role = create_test_role(neighbor_realm.id);

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&principal_realm.name, principal_realm.clone())
            .with_user_roles(admin.id, vec![admin_role])
            .with_successful_role_lookup(foreign_role.id, foreign_role.clone())
            .build();

        let result = service
            .get_role(identity, principal_realm.name.clone(), foreign_role.id)
            .await;

        assert!(matches!(result.unwrap_err(), CoreError::NotFound));
    }

    #[tokio::test]
    async fn test_delete_role_scoped_to_url_realm_returns_not_found() {
        let principal_realm = create_test_realm_with_name("principal");
        let neighbor_realm = create_test_realm_with_name("neighbor");
        let admin = create_test_user_with_realm(&principal_realm);
        let identity = Identity::User(admin.clone());

        let admin_role = create_test_role_with_params(
            principal_realm.id,
            "principal-manager",
            vec![Permissions::ManageRoles.name()],
            None,
        );

        let foreign_role = create_test_role(neighbor_realm.id);

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&principal_realm.name, principal_realm.clone())
            .with_user_roles(admin.id, vec![admin_role])
            .with_successful_role_lookup(foreign_role.id, foreign_role.clone())
            .build();

        let result = service
            .delete_role(identity, principal_realm.name.clone(), foreign_role.id)
            .await;

        assert!(matches!(result.unwrap_err(), CoreError::NotFound));
    }

    #[tokio::test]
    async fn test_update_role_records_a_security_event() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let role = create_test_role(realm.id);
        let admin_role = create_test_role_with_params(
            realm.id,
            "realm-admin",
            vec![Permissions::ManageRealm.name()],
            None,
        );
        let updated = create_test_role_with_params(realm.id, "renamed-role", vec![], None);

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .with_successful_role_update(role.id, updated.clone())
            .with_security_event_store()
            .with_role_webhook_notify()
            .build();

        let result = service
            .update_role(
                identity,
                UpdateRoleInput {
                    realm_name: realm.name.clone(),
                    role_id: role.id,
                    name: Some("renamed-role".to_string()),
                    description: None,
                    require_mfa: None,
                },
            )
            .await;

        let returned = assert_success(result);
        assert_eq!(returned.name, "renamed-role");
    }

    #[tokio::test]
    async fn test_update_role_permissions_records_a_security_event() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let role = create_test_role(realm.id);
        let admin_role = create_test_role_with_params(
            realm.id,
            "realm-admin",
            vec![Permissions::ManageRealm.name()],
            None,
        );
        let updated = create_test_role_with_params(
            realm.id,
            "test-role",
            vec![Permissions::ManageUsers.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .with_successful_role_permissions_update(role.id, updated.clone())
            .with_security_event_store()
            .with_role_webhook_notify()
            .build();

        let result = service
            .update_role_permissions(
                identity,
                realm.name.clone(),
                role.id,
                vec![Permissions::ManageUsers.name()],
            )
            .await;

        let returned = assert_success(result);
        assert_eq!(returned.permissions, vec![Permissions::ManageUsers.name()]);
    }

    #[tokio::test]
    async fn test_manage_users_reads_roles_but_cannot_create_them() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let user_admin = create_test_role_with_params(
            realm.id,
            "user-admin",
            vec![Permissions::ManageUsers.name()],
            None,
        );

        // No `with_successful_role_create`: the mock panics if creation is reached.
        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![user_admin])
            .build();

        let result = service
            .create_role(
                identity,
                CreateRoleInput {
                    realm_name: realm.name,
                    name: "anything".to_string(),
                    description: None,
                    permissions: vec![],
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "manage_users must no longer edit roles, got {result:?}"
        );
    }

    #[tokio::test]
    async fn test_manage_roles_cannot_grant_a_permission_it_does_not_hold() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let role = create_test_role(realm.id);
        let role_admin = create_test_role_with_params(
            realm.id,
            "role-admin",
            vec![Permissions::ManageRoles.name()],
            None,
        );

        // The guard runs before the role is even loaded: no lookup, no write.
        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![role_admin])
            .build();

        let result = service
            .update_role_permissions(
                identity,
                realm.name.clone(),
                role.id,
                vec![Permissions::ManageRealm.name()],
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "granting manage_realm without holding it is an escalation, got {result:?}"
        );
    }

    #[tokio::test]
    async fn test_manage_roles_grants_what_it_holds() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let identity = Identity::User(user.clone());
        let role = create_test_role(realm.id);
        let role_admin = create_test_role_with_params(
            realm.id,
            "role-admin",
            vec![
                Permissions::ManageRoles.name(),
                Permissions::ViewUsers.name(),
            ],
            None,
        );
        let updated = create_test_role_with_params(
            realm.id,
            "test-role",
            vec![Permissions::ViewUsers.name()],
            None,
        );

        let service = RoleServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![role_admin])
            .with_successful_role_lookup(role.id, role.clone())
            .with_successful_role_permissions_update(role.id, updated.clone())
            .with_security_event_store()
            .with_role_webhook_notify()
            .build();

        let result = service
            .update_role_permissions(
                identity,
                realm.name.clone(),
                role.id,
                vec![Permissions::ViewUsers.name()],
            )
            .await;

        let returned = assert_success(result);
        assert_eq!(returned.permissions, vec![Permissions::ViewUsers.name()]);
    }
}
