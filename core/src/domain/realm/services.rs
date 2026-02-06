use std::{collections::HashSet, sync::Arc};

use crate::domain::{
    abyss::identity_provider::{
        IdentityProviderRepository, entities::IdentityProviderPresentation,
    },
    authentication::value_objects::Identity,
    client::{ports::ClientRepository, value_objects::CreateClientRequest},
    common::{
        entities::app_errors::CoreError,
        generate_random_string,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::{
        entities::{Realm, RealmLoginSetting, RealmSetting},
        ports::{
            CreateRealmInput, CreateRealmWithUserInput, DeleteRealmInput, GetRealmInput,
            GetRealmSettingInput, RealmPolicy, RealmRepository, RealmService, UpdateRealmInput,
            UpdateRealmSettingInput,
        },
    },
    role::{
        entities::permission::Permissions, ports::RoleRepository, value_objects::CreateRoleRequest,
    },
    user::ports::{UserRepository, UserRoleRepository},
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct RealmServiceImpl<R, U, C, UR, RO, W, I>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    W: WebhookRepository,
    I: IdentityProviderRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) identity_provider_repository: Arc<I>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, RO, W, I> RealmServiceImpl<R, U, C, UR, RO, W, I>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    W: WebhookRepository,
    I: IdentityProviderRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        user_role_repository: Arc<UR>,
        role_repository: Arc<RO>,
        client_repository: Arc<C>,
        webhook_repository: Arc<W>,
        identity_provider_repository: Arc<I>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            user_role_repository,
            role_repository,
            client_repository,
            webhook_repository,
            identity_provider_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, RO, W, I> RealmService for RealmServiceImpl<R, U, C, UR, RO, W, I>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    RO: RoleRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    I: IdentityProviderRepository,
{
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm_master = self
            .realm_repository
            .get_by_name("master".to_string())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_master_id = realm_master.id;
        ensure_policy(
            self.policy.can_create_realm(&identity, &realm_master).await,
            "insufficient permissions",
        )?;

        let realm = self.realm_repository.create_realm(input.realm_name).await?;
        self.realm_repository
            .create_realm_settings(realm.id, "RS256".to_string())
            .await?;

        let name = format!("{}-realm", realm.name);

        let client = self
            .client_repository
            .create_client(CreateClientRequest::create_realm_system_client(
                realm_master_id,
                name.clone(),
            ))
            .await?;

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(client.id),
                description: None,
                name,
                permissions: vec![Permissions::ManageRealm.name()],
                realm_id: realm_master_id,
            })
            .await?;

        let user = match identity {
            Identity::User(u) => u,
            Identity::Client(c) => self.user_repository.get_by_client_id(c.id).await?,
        };

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await?;

        // Clients in the new realm
        self.client_repository
            .create_client(CreateClientRequest {
                client_id: "admin-cli".to_string(),
                client_type: "".to_string(),
                direct_access_grants_enabled: true,
                enabled: true,
                name: "admin-cli".to_string(),
                protocol: "openid-connect".to_string(),
                public_client: true,
                realm_id: realm.id,
                secret: None,
                service_account_enabled: false,
            })
            .await?;

        Ok(realm)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .create_realm(
                identity.clone(),
                CreateRealmInput {
                    realm_name: input.realm_name.clone(),
                },
            )
            .await?;

        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self.user_repository.get_by_client_id(client.id).await?,
        };

        let realm_master = self
            .realm_repository
            .get_by_name("master".to_string())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client_id = format!("{}-realm", input.realm_name);
        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id: realm_master.id,
                name: client_id.clone(),
                client_id,
                secret: Some(generate_random_string()),
                enabled: true,
                protocol: "openid-connect".to_string(),
                public_client: true,
                service_account_enabled: false,
                direct_access_grants_enabled: false,
                client_type: "public".into(),
            })
            .await?;

        // Create role for client
        let permissions = Permissions::to_names(&[
            Permissions::ManageRealm,
            Permissions::ManageClients,
            Permissions::ManageRoles,
            Permissions::ManageUsers,
        ]);

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(client.id),
                name: format!("{}-realm-admin", input.realm_name),
                permissions,
                realm_id: realm_master.id,
                description: Some(format!("role for manage realm {}", input.realm_name)),
            })
            .await?;

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await?;

        Ok(realm)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        if !realm.can_delete() {
            return Err(CoreError::CannotDeleteMasterRealm);
        }

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_realm(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::RealmCreated, realm_id.into(), Some(realm)),
            )
            .await?;

        self.realm_repository
            .delete_by_name(input.realm_name)
            .await?;

        Ok(())
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_realm(&identity, &realm).await,
            "insufficient permissions",
        )?;

        Ok(realm)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> Result<RealmSetting, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_view_realm(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let realm_setting = self
            .realm_repository
            .get_realm_settings(realm_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        Ok(realm_setting)
    }

    #[instrument(
        skip(self, identity),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
        )
    )]
    async fn get_realms_by_user(&self, identity: Identity) -> Result<Vec<Realm>, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self.user_repository.get_by_client_id(client.id).await?,
        };

        let realm = user.realm.clone().ok_or(CoreError::InternalServerError)?;
        self.realm_repository
            .get_by_name(realm.name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let user_roles = self.user_role_repository.get_user_roles(user.id).await?;

        let realms = self.realm_repository.fetch_realm().await?;

        let mut user_realms: Vec<Realm> = Vec::new();

        for realm in realms {
            let client_name = format!("{}-realm", realm.name);

            let client_roles = user_roles
                .iter()
                .filter(|role| role.client.is_some())
                .filter(|role| role.client.as_ref().unwrap().name == client_name)
                .collect::<Vec<_>>();

            let mut permissions = HashSet::new();

            for role in client_roles {
                let role_permissions = role
                    .permissions
                    .iter()
                    .filter_map(|perm_str| Permissions::from_name(perm_str))
                    .collect::<HashSet<Permissions>>();

                permissions.extend(role_permissions);
            }

            let has_access = Permissions::has_one_of_permissions(
                &permissions.iter().cloned().collect::<Vec<Permissions>>(),
                &[
                    Permissions::QueryRealms,
                    Permissions::ManageRealm,
                    Permissions::ViewRealm,
                ],
            );

            if has_access {
                user_realms.push(realm.clone());
            }
        }

        Ok(user_realms)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_realm(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let realm = self
            .realm_repository
            .update_realm(input.realm_name, input.name)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RealmUpdated,
                    realm_id.into(),
                    Some(realm.clone()),
                ),
            )
            .await?;

        Ok(realm)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> Result<Realm, CoreError> {
        let mut realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_realm(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let realm_setting = self
            .realm_repository
            .update_realm_setting(
                realm.id,
                input.algorithm,
                input.user_registration_enabled,
                input.forgot_password_enabled,
                input.remember_me_enabled,
            )
            .await?;

        self.webhook_repository
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::RealmSettingsUpdated,
                    realm.id.into(),
                    Some(realm_setting.clone()),
                ),
            )
            .await?;

        realm.settings = Some(realm_setting);

        Ok(realm)
    }

    #[instrument(
        skip(self, realm_name),
        fields(
            realm.name = %realm_name,
        )
    )]
    async fn get_login_settings(&self, realm_name: String) -> Result<RealmLoginSetting, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let mut settings: RealmLoginSetting = self
            .realm_repository
            .get_realm_settings(realm.id)
            .await?
            .ok_or(CoreError::NotFound)?
            .into();

        let idp: Vec<IdentityProviderPresentation> = self
            .identity_provider_repository
            .list_identity_providers_by_realm(realm.id, Some(true))
            .await?
            .into_iter()
            .map(|i| IdentityProviderPresentation::new(i, &realm.name))
            .collect();

        settings.identity_providers = idp;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        abyss::identity_provider::ports::MockIdentityProviderRepository,
        client::ports::MockClientRepository,
        common::services::tests::{
            create_test_realm_with_name, create_test_user_identity_with_realm,
        },
        realm::{entities::RealmId, ports::MockRealmRepository},
        role::ports::MockRoleRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
        webhook::ports::MockWebhookRepository,
    };

    struct RealmServiceTestBuilder {
        realm_repo: Arc<MockRealmRepository>,
        role_repo: Arc<MockRoleRepository>,
        webhook_repo: Arc<MockWebhookRepository>,
        user_role_repo: Arc<MockUserRoleRepository>,
        client_repo: Arc<MockClientRepository>,
        user_repo: Arc<MockUserRepository>,
        identity_provider: Arc<MockIdentityProviderRepository>,
    }

    impl RealmServiceTestBuilder {
        pub fn new() -> Self {
            let realm_repo = Arc::new(MockRealmRepository::new());
            let role_repo = Arc::new(MockRoleRepository::new());
            let webhook_repo = Arc::new(MockWebhookRepository::new());
            let user_role_repo = Arc::new(MockUserRoleRepository::new());
            let client_repo = Arc::new(MockClientRepository::new());
            let user_repo = Arc::new(MockUserRepository::new());
            let identity_provider = Arc::new(MockIdentityProviderRepository::new());

            Self {
                realm_repo,
                role_repo,
                webhook_repo,
                user_role_repo,
                client_repo,
                user_repo,
                identity_provider,
            }
        }

        fn with_master_realm(mut self, master_realm: Realm) -> Self {
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(mockall::predicate::eq("master".to_string()))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(master_realm)) }));
            self
        }

        fn with_master_realm_twice(mut self, master_realm: Realm) -> Self {
            let master_realm_clone = master_realm.clone();
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(mockall::predicate::eq("master".to_string()))
                .times(2)
                .returning(move |_| {
                    let realm = master_realm_clone.clone();
                    Box::pin(async move { Ok(Some(realm)) })
                });
            self
        }

        fn with_created_realm(mut self, realm_name: String, new_realm: Realm) -> Self {
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_create_realm()
                .with(mockall::predicate::eq(realm_name))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(new_realm) }));
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

        fn with_realm_settings(mut self, realm_id: RealmId) -> Self {
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_create_realm_settings()
                .with(
                    mockall::predicate::eq(realm_id),
                    mockall::predicate::eq("RS256".to_string()),
                )
                .times(1)
                .return_once(move |realm_id, algorithm| {
                    Box::pin(async move { Ok(RealmSetting::new(realm_id, Some(algorithm))) })
                });
            self
        }

        fn with_system_client(mut self, master_realm_id: RealmId) -> Self {
            Arc::get_mut(&mut self.client_repo)
                .unwrap()
                .expect_create_client()
                .withf(move |req| {
                    // Vérifier que c'est le client système
                    req.name.contains("-realm") && req.realm_id == master_realm_id
                })
                .times(1)
                .return_once(move |req| {
                    Box::pin(async move {
                        Ok(crate::domain::client::entities::Client::new(
                            crate::domain::client::entities::ClientConfig {
                                realm_id: req.realm_id,
                                name: req.name.clone(),
                                client_id: req.client_id.clone(),
                                secret: req.secret.clone(),
                                enabled: true,
                                protocol: "openid-connect".to_string(),
                                public_client: req.public_client,
                                service_account_enabled: req.service_account_enabled,
                                client_type: req.client_type.clone(),
                                direct_access_grants_enabled: Some(
                                    req.direct_access_grants_enabled,
                                ),
                            },
                        ))
                    })
                });
            self
        }

        fn with_admin_cli_client(mut self, new_realm_id: RealmId) -> Self {
            Arc::get_mut(&mut self.client_repo)
                .unwrap()
                .expect_create_client()
                .withf(move |req| req.client_id == "admin-cli" && req.realm_id == new_realm_id)
                .times(1)
                .return_once(move |req| {
                    Box::pin(async move {
                        Ok(crate::domain::client::entities::Client::new(
                            crate::domain::client::entities::ClientConfig {
                                realm_id: req.realm_id,
                                name: req.name.clone(),
                                client_id: req.client_id.clone(),
                                secret: None,
                                enabled: true,
                                protocol: "openid-connect".to_string(),
                                public_client: true,
                                service_account_enabled: false,
                                client_type: req.client_type.clone(),
                                direct_access_grants_enabled: Some(true),
                            },
                        ))
                    })
                });
            self
        }

        fn with_role_creation(mut self, master_realm_id: RealmId) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_create()
                .withf(move |req| {
                    req.realm_id == master_realm_id
                        && req.permissions.contains(&"manage_realm".to_string())
                        && req.name.ends_with("-realm")
                })
                .times(1)
                .return_once(move |req| {
                    Box::pin(async move {
                        Ok(crate::domain::role::entities::Role {
                            id: uuid::Uuid::new_v4(),
                            name: req.name,
                            description: req.description,
                            permissions: req.permissions,
                            realm_id: req.realm_id,
                            client_id: req.client_id,
                            client: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        })
                    })
                });

            self
        }

        fn with_assign_role(mut self) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_assign_role()
                .withf(|_, _| true)
                .times(1)
                .return_once(|_, _| Box::pin(async move { Ok(()) }));
            self
        }

        fn with_admin_role_creation(mut self, master_realm_id: RealmId) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_create()
                .withf(move |req| {
                    req.realm_id == master_realm_id
                        && req.name.ends_with("-realm-admin")
                        && req.permissions.contains(&"manage_realm".to_string())
                        && req.permissions.contains(&"manage_clients".to_string())
                        && req.permissions.contains(&"manage_roles".to_string())
                        && req.permissions.contains(&"manage_users".to_string())
                })
                .times(1)
                .return_once(move |req| {
                    Box::pin(async move {
                        Ok(crate::domain::role::entities::Role {
                            id: uuid::Uuid::new_v4(),
                            name: req.name,
                            description: req.description,
                            permissions: req.permissions,
                            realm_id: req.realm_id,
                            client_id: req.client_id,
                            client: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        })
                    })
                });
            self
        }

        fn build(
            self,
        ) -> RealmServiceImpl<
            MockRealmRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
            MockRoleRepository,
            MockWebhookRepository,
            MockIdentityProviderRepository,
        > {
            let policy = FerriskeyPolicy::new(
                self.user_repo.clone(),
                self.client_repo.clone(),
                self.user_role_repo.clone(),
            );
            RealmServiceImpl::new(
                self.realm_repo,
                self.user_repo,
                self.user_role_repo,
                self.role_repo,
                self.client_repo,
                self.webhook_repo,
                self.identity_provider,
                Arc::new(policy),
            )
        }
    }

    #[tokio::test]
    async fn test_create_realm() -> Result<(), CoreError> {
        let realm_name = "realm_test";
        let master_realm = create_test_realm_with_name("master");
        let identity = create_test_user_identity_with_realm(&master_realm);

        // Extract user for mocking permissions
        let user = match &identity {
            Identity::User(u) => u,
            _ => panic!("Expected user identity"),
        };

        // Create role with ManageRealm permission for the user
        let admin_role = crate::domain::role::entities::Role {
            id: uuid::Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![
                crate::domain::role::entities::permission::Permissions::ManageRealm.name(),
            ],
            realm_id: master_realm.id,
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let input = CreateRealmInput {
            realm_name: "realm_test".to_string(),
        };

        // Create the new realm that will be returned
        let new_realm = create_test_realm_with_name(realm_name);

        let service = RealmServiceTestBuilder::new()
            .with_master_realm(master_realm.clone())
            .with_user_permissions(user.id, vec![admin_role])
            .with_created_realm("realm_test".to_string(), new_realm.clone())
            .with_realm_settings(new_realm.id)
            .with_system_client(master_realm.id)
            .with_role_creation(master_realm.id)
            .with_assign_role()
            .with_admin_cli_client(new_realm.id)
            .build();

        let created_realm = service.create_realm(identity, input).await?;
        assert_eq!(created_realm.name, realm_name);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_realm_with_user() -> Result<(), CoreError> {
        let realm_name = "realm_test";

        let master_realm = create_test_realm_with_name("master");
        let identity = create_test_user_identity_with_realm(&master_realm);

        // Extract user for mocking permissions
        let user = match &identity {
            Identity::User(u) => u,
            _ => panic!("Expected user identity"),
        };

        // Create role with ManageRealm permission for the user
        let admin_role = crate::domain::role::entities::Role {
            id: uuid::Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![
                crate::domain::role::entities::permission::Permissions::ManageRealm.name(),
            ],
            realm_id: master_realm.id,
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let input = CreateRealmWithUserInput {
            realm_name: "realm_test".to_string(),
            user: user.clone(),
        };

        // Create the new realm that will be returned
        let new_realm = create_test_realm_with_name(realm_name);

        let service = RealmServiceTestBuilder::new()
            .with_master_realm_twice(master_realm.clone())
            .with_user_permissions(user.id, vec![admin_role])
            .with_created_realm("realm_test".to_string(), new_realm.clone())
            .with_realm_settings(new_realm.id)
            .with_system_client(master_realm.id)
            .with_role_creation(master_realm.id)
            .with_assign_role()
            .with_admin_cli_client(new_realm.id)
            .with_system_client(master_realm.id)
            .with_admin_role_creation(master_realm.id)
            .with_assign_role()
            .build();

        let created_realm = service.create_realm_with_user(identity, input).await?;
        assert_eq!(created_realm.name, realm_name);

        Ok(())
    }
}
