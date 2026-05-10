use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    realm_branding::{
        entities::{BrandingConfig, RealmBranding},
        ports::{
            GetBrandingInput, RealmBrandingPolicy, RealmBrandingRepository, RealmBrandingService,
            UpdateBrandingInput,
        },
    },
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct RealmBrandingServiceImpl<R, U, C, UR, RB>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RB: RealmBrandingRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) realm_branding_repository: Arc<RB>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, RB> RealmBrandingServiceImpl<R, U, C, UR, RB>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RB: RealmBrandingRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        realm_branding_repository: Arc<RB>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            realm_branding_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, RB> RealmBrandingService for RealmBrandingServiceImpl<R, U, C, UR, RB>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RB: RealmBrandingRepository,
{
    async fn get_branding(
        &self,
        identity: Identity,
        input: GetBrandingInput,
    ) -> Result<BrandingConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_branding(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let stored = self
            .realm_branding_repository
            .get_by_realm(realm.id.into())
            .await?;

        Ok(stored.map(|b| b.config).unwrap_or_default())
    }

    async fn update_branding(
        &self,
        identity: Identity,
        input: UpdateBrandingInput,
    ) -> Result<RealmBranding, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_branding(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.realm_branding_repository
            .upsert(realm.id.into(), input.config)
            .await
    }

    async fn get_public_branding(
        &self,
        input: GetBrandingInput,
    ) -> Result<BrandingConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let stored = self
            .realm_branding_repository
            .get_by_realm(realm.id.into())
            .await?;

        Ok(stored.map(|b| b.config).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        realm::{entities::Realm, ports::MockRealmRepository},
        realm_branding::ports::MockRealmBrandingRepository,
        role::entities::Role,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRoleRepository},
        },
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn test_realm() -> Realm {
        Realm {
            id: Uuid::new_v4().into(),
            name: "test-realm".to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            username: "admin".to_string(),
            firstname: Some("Admin".to_string()),
            lastname: Some("User".to_string()),
            email: Some("admin@test.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn admin_role(realm: &Realm) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec!["manage_realm".to_string()],
            realm_id: realm.id,
            client_id: None,
            client: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn empty_role(realm: &Realm) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "viewer".to_string(),
            description: None,
            permissions: vec![],
            realm_id: realm.id,
            client_id: None,
            client: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_branding(realm: &Realm, config: BrandingConfig) -> RealmBranding {
        RealmBranding {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        branding_repo: MockRealmBrandingRepository,
    ) -> RealmBrandingServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockRealmBrandingRepository,
    > {
        let client_repo = MockClientRepository::new();
        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));
        RealmBrandingServiceImpl::new(Arc::new(realm_repo), Arc::new(branding_repo), policy)
    }

    #[tokio::test]
    async fn update_branding_forbidden_when_policy_denies() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        let user_clone = user.clone();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user_clone.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let realm_for_roles = realm.clone();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = realm_for_roles.clone();
            Box::pin(async move { Ok(vec![empty_role(&r)]) })
        });

        let branding_repo = MockRealmBrandingRepository::new();

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .update_branding(
                Identity::User(user),
                UpdateBrandingInput {
                    realm_name: realm.name.clone(),
                    config: BrandingConfig::default(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn update_branding_upserts_when_allowed() {
        let realm = test_realm();
        let user = test_user(&realm);
        let mut new_config = BrandingConfig::default();
        new_config.colors.primary_button = "#ff00aa".to_string();

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        let user_clone = user.clone();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user_clone.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let realm_for_roles = realm.clone();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = realm_for_roles.clone();
            Box::pin(async move { Ok(vec![admin_role(&r)]) })
        });

        let mut branding_repo = MockRealmBrandingRepository::new();
        let realm_for_repo = realm.clone();
        branding_repo
            .expect_upsert()
            .returning(move |_realm_id, config| {
                let stored = stored_branding(&realm_for_repo, config);
                Box::pin(async move { Ok(stored) })
            });

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .update_branding(
                Identity::User(user),
                UpdateBrandingInput {
                    realm_name: realm.name.clone(),
                    config: new_config.clone(),
                },
            )
            .await
            .expect("upsert should succeed");

        assert_eq!(result.config, new_config);
    }

    #[tokio::test]
    async fn get_branding_returns_default_when_repo_empty() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        let user_clone = user.clone();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user_clone.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let realm_for_roles = realm.clone();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = realm_for_roles.clone();
            Box::pin(async move { Ok(vec![admin_role(&r)]) })
        });

        let mut branding_repo = MockRealmBrandingRepository::new();
        branding_repo
            .expect_get_by_realm()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .get_branding(
                Identity::User(user),
                GetBrandingInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await
            .expect("get should succeed");

        assert_eq!(result, BrandingConfig::default());
    }

    #[tokio::test]
    async fn get_branding_returns_stored_config() {
        let realm = test_realm();
        let user = test_user(&realm);
        let mut stored_config = BrandingConfig::default();
        stored_config.widget.logo_height = 80;

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        let user_clone = user.clone();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user_clone.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let realm_for_roles = realm.clone();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = realm_for_roles.clone();
            Box::pin(async move { Ok(vec![admin_role(&r)]) })
        });

        let mut branding_repo = MockRealmBrandingRepository::new();
        let realm_for_repo = realm.clone();
        let stored_clone = stored_config.clone();
        branding_repo.expect_get_by_realm().returning(move |_| {
            let stored = stored_branding(&realm_for_repo, stored_clone.clone());
            Box::pin(async move { Ok(Some(stored)) })
        });

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .get_branding(
                Identity::User(user),
                GetBrandingInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await
            .expect("get should succeed");

        assert_eq!(result, stored_config);
    }

    #[tokio::test]
    async fn get_public_branding_skips_policy_and_returns_default_when_empty() {
        let realm = test_realm();

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();

        let mut branding_repo = MockRealmBrandingRepository::new();
        branding_repo
            .expect_get_by_realm()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .get_public_branding(GetBrandingInput {
                realm_name: realm.name.clone(),
            })
            .await
            .expect("public get should succeed");

        assert_eq!(result, BrandingConfig::default());
    }

    #[tokio::test]
    async fn get_public_branding_returns_stored_config_without_auth() {
        let realm = test_realm();
        let mut stored_config = BrandingConfig::default();
        stored_config.colors.primary_button = "#abcdef".to_string();

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();

        let mut branding_repo = MockRealmBrandingRepository::new();
        let realm_for_repo = realm.clone();
        let stored_clone = stored_config.clone();
        branding_repo.expect_get_by_realm().returning(move |_| {
            let stored = stored_branding(&realm_for_repo, stored_clone.clone());
            Box::pin(async move { Ok(Some(stored)) })
        });

        let service = build_service(realm_repo, user_repo, user_role_repo, branding_repo);

        let result = service
            .get_public_branding(GetBrandingInput {
                realm_name: realm.name.clone(),
            })
            .await
            .expect("public get should succeed");

        assert_eq!(result, stored_config);
    }
}
