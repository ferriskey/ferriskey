use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    seawatch::{EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    user::{
        entities::{
            AssignRoleInput, CreateUserInput, GetUserInput, RequiredAction, ResetPasswordInput,
            UnassignRoleInput, UpdateUserInput, User,
        },
        ports::{
            UserPolicy, UserRepository, UserRequiredActionRepository, UserRoleRepository,
            UserService,
        },
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use serde_json::json;

pub mod user_role_service;

#[derive(Clone, Debug)]
pub struct UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) security_event_repository: Arc<SE>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CR, H, RO, URA, W, SE> UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        credential_repository: Arc<CR>,
        hasher_repository: Arc<H>,
        user_role_repository: Arc<UR>,
        role_repository: Arc<RO>,
        user_required_action_repository: Arc<URA>,
        webhook_repository: Arc<W>,
        security_event_repository: Arc<SE>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            user_role_repository,
            role_repository,
            user_required_action_repository,
            webhook_repository,
            security_event_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CR, H, RO, URA, W, SE> UserService
    for UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
{
    async fn delete_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.user_repository.get_by_id(user_id).await?;

        let count = self
            .user_repository
            .delete_user(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserDeleted,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::UserDeleted, realm_id.into(), Some(user)),
            )
            .await?;

        Ok(count)
    }

    async fn reset_password(
        &self,
        identity: Identity,
        input: ResetPasswordInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let password_credential = self
            .credential_repository
            .get_password_credential(input.user_id)
            .await;

        if password_credential.is_ok() {
            self.credential_repository
                .delete_password_credential(input.user_id)
                .await
                .map_err(|_| CoreError::DeletePasswordCredentialError)?;
        }

        let hash_result = self
            .hasher_repository
            .hash_password(&input.password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(
                input.user_id,
                "password".into(),
                hash_result,
                "".into(),
                input.temporary,
            )
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::PasswordReset,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), input.user_id, None),
            )
            .await?;

        // @TODO: webhook call action

        Ok(())
    }

    async fn update_user(
        &self,
        identity: Identity,
        input: UpdateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        let user = self
            .user_repository
            .update_user(
                input.user_id,
                UpdateUserRequest {
                    email: input.email,
                    email_verified: input.email_verified.unwrap_or(false),
                    enabled: input.enabled,
                    firstname: input.firstname,
                    lastname: input.lastname,
                    required_actions: None,
                },
            )
            .await?;

        if let Some(required_actions) = input.required_actions {
            self.user_required_action_repository
                .clear_required_actions(user.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            for action in required_actions {
                let required_action: RequiredAction =
                    RequiredAction::try_from(action).map_err(|_| CoreError::InternalServerError)?;
                self.user_required_action_repository
                    .add_required_action(user.id, required_action)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
            }
        }

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserUpdated,
                    realm_id.into(),
                    Some(user.clone()),
                ),
            )
            .await?;

        Ok(user)
    }

    async fn get_users(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<User>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        self.user_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let role = self.role_repository.get_by_id(input.role_id).await?;
        self.user_role_repository
            .assign_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::RoleAssigned,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target(
                    "user".to_string(),
                    input.user_id,
                    role.as_ref().map(|value| value.name.clone()),
                ),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserRoleAssigned,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(())
    }

    async fn bulk_delete_users(
        &self,
        identity: Identity,
        input: crate::domain::user::entities::BulkDeleteUsersInput,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let count = self
            .user_repository
            .bulk_delete_user(input.ids.clone())
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserDeleted,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_details(json!({ "user_ids": input.ids })),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserBulkDeleted,
                    realm_id.into(),
                    Some(input.ids),
                ),
            )
            .await?;

        Ok(count)
    }

    async fn create_user(
        &self,
        identity: Identity,
        input: CreateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let mut user = self
            .user_repository
            .create_user(CreateUserRequest {
                client_id: None,
                realm_id,
                username: input.username,
                firstname: input.firstname,
                lastname: input.lastname,
                email: input.email,
                email_verified: input.email_verified.unwrap_or(false),
                enabled: true,
            })
            .await?;

        user.realm = Some(realm);

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserCreated,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserCreated,
                    realm_id.into(),
                    Some(user.clone()),
                ),
            )
            .await?;

        Ok(user)
    }

    async fn get_user(&self, identity: Identity, input: GetUserInput) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.user_repository
            .get_by_id(input.user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn unassign_role(
        &self,
        identity: Identity,
        input: UnassignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let role = self.role_repository.get_by_id(input.role_id).await?;

        self.user_role_repository
            .revoke_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::RoleUnassigned,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target(
                    "user".to_string(),
                    input.user_id,
                    role.as_ref().map(|value| value.name.clone()),
                ),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserUpdated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        common::services::tests::{
            create_test_realm_with_name, create_test_user_identity_with_realm,
            create_test_user_with_params_and_realm,
        },
        credential::ports::MockCredentialRepository,
        crypto::ports::MockHasherRepository,
        realm::{entities::Realm, ports::MockRealmRepository},
        role::ports::MockRoleRepository,
        seawatch::ports::MockSecurityEventRepository,
        user::ports::{
            MockUserRepository, MockUserRequiredActionRepository, MockUserRoleRepository,
        },
        webhook::{entities::webhook_payload::WebhookPayload, ports::MockWebhookRepository},
    };

    struct UserServiceTestBuilder {
        realm_repo: Arc<MockRealmRepository>,
        user_repo: Arc<MockUserRepository>,
        credential_repo: Arc<MockCredentialRepository>,
        hasher_repo: Arc<MockHasherRepository>,
        user_role_repo: Arc<MockUserRoleRepository>,
        role_repo: Arc<MockRoleRepository>,
        user_required_action_repo: Arc<MockUserRequiredActionRepository>,
        webhook_repo: Arc<MockWebhookRepository>,
        client_repo: Arc<MockClientRepository>,
        security_event_repo: Arc<MockSecurityEventRepository>,
    }

    impl UserServiceTestBuilder {
        pub fn new() -> Self {
            Self {
                realm_repo: Arc::new(MockRealmRepository::new()),
                user_repo: Arc::new(MockUserRepository::new()),
                credential_repo: Arc::new(MockCredentialRepository::new()),
                hasher_repo: Arc::new(MockHasherRepository::new()),
                user_role_repo: Arc::new(MockUserRoleRepository::new()),
                role_repo: Arc::new(MockRoleRepository::new()),
                user_required_action_repo: Arc::new(MockUserRequiredActionRepository::new()),
                webhook_repo: Arc::new(MockWebhookRepository::new()),
                client_repo: Arc::new(MockClientRepository::new()),
                security_event_repo: Arc::new(MockSecurityEventRepository::new()),
            }
        }

        fn with_realm(mut self, realm_name: String, realm: Realm) -> Self {
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(mockall::predicate::eq(realm_name))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(realm)) }));
            self
        }

        fn with_user_permissions(
            mut self,
            user_id: uuid::Uuid,
            roles: Vec<crate::domain::role::entities::Role>,
        ) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_get_user_roles()
                .with(mockall::predicate::eq(user_id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(roles) }));
            self
        }

        fn with_create_user_success(mut self, created_user: User) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_create_user()
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(created_user) }));
            Arc::get_mut(&mut self.security_event_repo)
                .unwrap()
                .expect_store_event()
                .times(1)
                .return_once(|_| Box::pin(async move { Ok(()) }));
            self
        }

        fn with_create_user_email_exists(mut self) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_create_user()
                .times(1)
                .return_once(move |_| Box::pin(async move { Err(CoreError::EmailAlreadyExists) }));
            self
        }

        fn with_update_user_success(mut self, user_id: uuid::Uuid, updated_user: User) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_update_user()
                .with(
                    mockall::predicate::eq(user_id),
                    mockall::predicate::always(),
                )
                .times(1)
                .return_once(move |_, _| Box::pin(async move { Ok(updated_user) }));
            self
        }

        fn with_update_user_email_exists(mut self, user_id: uuid::Uuid) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_update_user()
                .with(
                    mockall::predicate::eq(user_id),
                    mockall::predicate::always(),
                )
                .times(1)
                .return_once(move |_, _| {
                    Box::pin(async move { Err(CoreError::EmailAlreadyExists) })
                });
            self
        }

        fn with_webhook_notify(mut self) -> Self {
            Arc::get_mut(&mut self.webhook_repo)
                .unwrap()
                .expect_notify::<User>()
                .times(1)
                .return_once(|_, _: WebhookPayload<User>| Box::pin(async move { Ok(()) }));
            self
        }

        fn build(
            self,
        ) -> UserServiceImpl<
            MockRealmRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
            MockCredentialRepository,
            MockHasherRepository,
            MockRoleRepository,
            MockUserRequiredActionRepository,
            MockWebhookRepository,
            MockSecurityEventRepository,
        > {
            use crate::domain::common::policies::FerriskeyPolicy;

            let policy = FerriskeyPolicy::new(
                self.user_repo.clone(),
                self.client_repo.clone(),
                self.user_role_repo.clone(),
            );

            UserServiceImpl::new(
                self.realm_repo,
                self.user_repo,
                self.credential_repo,
                self.hasher_repo,
                self.user_role_repo,
                self.role_repo,
                self.user_required_action_repo,
                self.webhook_repo,
                self.security_event_repo,
                Arc::new(policy),
            )
        }
    }

    fn create_admin_role(realm: &Realm) -> crate::domain::role::entities::Role {
        crate::domain::role::entities::Role {
            id: uuid::Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![
                crate::domain::role::entities::permission::Permissions::ManageUsers.name(),
            ],
            realm_id: realm.id,
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_user_with_duplicate_email_in_same_realm_fails() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        // Repository returns EmailAlreadyExists when constraint is violated
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_create_user_email_exists()
            .build();

        let input = CreateUserInput {
            realm_name: "test-realm".to_string(),
            username: "new_user".to_string(),
            firstname: "New".to_string(),
            lastname: "User".to_string(),
            email: "test@example.com".to_string(),
            email_verified: Some(false),
        };

        let result = service.create_user(identity, input).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn test_create_user_with_unique_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let new_user = create_test_user_with_params_and_realm(
            &realm,
            "new_user",
            "unique@example.com".to_string(),
            true,
        );

        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_create_user_success(new_user.clone())
            .with_webhook_notify()
            .build();

        let input = CreateUserInput {
            realm_name: "test-realm".to_string(),
            username: "new_user".to_string(),
            firstname: "New".to_string(),
            lastname: "User".to_string(),
            email: "unique@example.com".to_string(),
            email_verified: Some(false),
        };

        let result = service.create_user(identity, input).await;

        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.email, "unique@example.com");
    }

    #[tokio::test]
    async fn test_update_user_with_another_users_email_fails() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "original@example.com".to_string(),
            true,
        );

        // Repository returns EmailAlreadyExists when constraint is violated
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_update_user_email_exists(user_to_update.id)
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: user_to_update.id,
            firstname: "Updated".to_string(),
            lastname: "User".to_string(),
            email: "taken@example.com".to_string(), // Email belongs to another user
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn test_update_user_keeping_own_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let mut user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "myemail@example.com".to_string(),
            true,
        );

        let update_user_id = user_to_update.id;
        user_to_update.firstname = "Updated".to_string();

        // Keeping own email doesn't violate the constraint
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_update_user_success(update_user_id, user_to_update.clone())
            .with_webhook_notify()
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: update_user_id,
            firstname: "Updated".to_string(),
            lastname: "User".to_string(),
            email: "myemail@example.com".to_string(), // Same email as before
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email, "myemail@example.com");
        assert_eq!(updated_user.firstname, "Updated");
    }

    #[tokio::test]
    async fn test_update_user_with_new_unique_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let mut user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "old@example.com".to_string(),
            true,
        );

        let update_user_id = user_to_update.id;
        user_to_update.email = "newemail@example.com".to_string();

        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_update_user_success(update_user_id, user_to_update.clone())
            .with_webhook_notify()
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: update_user_id,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            email: "newemail@example.com".to_string(), // New unique email
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email, "newemail@example.com");
    }
}
