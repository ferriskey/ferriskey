use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    role::{
        entities::{GetUserRolesInput, Role, UpdateRoleInput},
        ports::{RolePolicy, RoleRepository, RoleService},
        value_objects::{UpdateRolePermissionsRequest, UpdateRoleRequest},
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
    pub(crate) user_role_repository: Arc<UR>,

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
        user_role_repository: Arc<UR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            role_repository,
            security_event_repository,
            webhook_repository,
            user_role_repository,
            policy,
        }
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
    async fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_delete_role(identity.clone(), realm).await,
            "insufficient permissions",
        )?;

        let role = self.role_repository.get_by_id(role_id).await?;
        self.role_repository.delete_by_id(role_id).await?;

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
                WebhookPayload::new(WebhookTrigger::RoleDeleted, realm_id.into(), Some(role)),
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
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .ok_or(CoreError::NotFound)
    }

    async fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn get_user_roles(
        &self,
        identity: Identity,
        input: GetUserRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_role_repository
            .get_user_roles(input.user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_by_id(
                input.role_id,
                UpdateRoleRequest {
                    description: input.description,
                    name: input.name,
                },
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

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
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_permissions_by_id(role_id, UpdateRolePermissionsRequest { permissions })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

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
    use crate::domain::{
        common::entities::app_errors::CoreError,
        realm::entities::{Realm, RealmId, RealmSetting},
    };

    mockall::mock! {
        pub RealmRepository {}
        impl crate::domain::realm::ports::RealmRepository for RealmRepository {
            fn fetch_realm(
                &self,
            ) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;
            fn get_by_name(
                &self,
                name: String,
            ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;
            fn create_realm(
                &self,
                name: String,
            ) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn update_realm(
                &self,
                realm_name: String,
                name: String,
            ) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn delete_by_name(
                &self,
                name: String,
            ) -> impl Future<Output = Result<(), CoreError>> + Send;
            fn create_realm_settings(
                &self,
                realm_id: RealmId,
                algorithm: String,
            ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn update_realm_setting(
                &self,
                realm_id: RealmId,
                algorithm: Option<String>,
                user_registration_enabled: Option<bool>,
                forgot_password_enabled: Option<bool>,
                remember_me_enabled: Option<bool>,
            ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn get_realm_settings(
                &self,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Option<RealmSetting>, CoreError>> + Send;
        }
    }

    use mockall::predicate::*;
    use uuid::Uuid;

    use crate::domain::{
        authentication::value_objects::Identity,
        client::{entities::Client, ports::MockClientRepository},
        common::{
            policies::FerriskeyPolicy,
            services::tests::{
                assert_success, create_test_realm, create_test_realm_with_name, create_test_role,
                create_test_role_with_params, create_test_user, create_test_user_with_realm,
            },
        },
        role::{
            entities::{Role, permission::Permissions},
            ports::{MockRoleRepository, RoleService},
            services::RoleServiceImpl,
        },
        seawatch::ports::MockSecurityEventRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
        webhook::ports::MockWebhookRepository,
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
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(eq(realm_name.to_string()))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(realm)) }));

            self
        }

        fn with_successful_role_lookup(mut self, role_id: Uuid, role: Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_get_by_id()
                .with(eq(role_id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(role)) }));
            self
        }

        fn with_user_roles(mut self, user_id: Uuid, roles: Vec<Role>) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_get_user_roles()
                .with(eq(user_id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(roles) }));
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
                .return_once(move |_, _| Box::pin(async move { Ok(client) }));
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
                self.user_role_repo,
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
}
