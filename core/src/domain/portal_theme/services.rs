use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    portal_theme::{
        entities::{PortalTheme, PortalThemeConfig},
        ports::{
            GetThemeInput, PortalThemePolicy, PortalThemeRepository, PortalThemeService,
            UpdateThemeInput,
        },
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct PortalThemeServiceImpl<R, U, C, UR, PT>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PT: PortalThemeRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) portal_theme_repository: Arc<PT>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, PT> PortalThemeServiceImpl<R, U, C, UR, PT>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PT: PortalThemeRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        portal_theme_repository: Arc<PT>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            portal_theme_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, PT> PortalThemeService for PortalThemeServiceImpl<R, U, C, UR, PT>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PT: PortalThemeRepository,
{
    async fn get_theme(
        &self,
        identity: Identity,
        input: GetThemeInput,
    ) -> Result<PortalThemeConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let stored = self
            .portal_theme_repository
            .get_by_realm(realm.id.into())
            .await?;

        Ok(stored.map(|b| b.config).unwrap_or_default())
    }

    async fn update_theme(
        &self,
        identity: Identity,
        input: UpdateThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_theme_repository
            .upsert(realm.id.into(), input.config)
            .await
    }

    async fn get_public_theme(&self, input: GetThemeInput) -> Result<PortalThemeConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let stored = self
            .portal_theme_repository
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
        portal_theme::ports::MockPortalThemeRepository,
        realm::{entities::Realm, ports::MockRealmRepository},
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

    fn stored_theme(realm: &Realm, config: PortalThemeConfig) -> PortalTheme {
        PortalTheme {
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
        theme_repo: MockPortalThemeRepository,
    ) -> PortalThemeServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockPortalThemeRepository,
    > {
        let client_repo = MockClientRepository::new();
        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));
        PortalThemeServiceImpl::new(Arc::new(realm_repo), Arc::new(theme_repo), policy)
    }

    #[tokio::test]
    async fn update_theme_forbidden_when_policy_denies() {
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

        let theme_repo = MockPortalThemeRepository::new();

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .update_theme(
                Identity::User(user),
                UpdateThemeInput {
                    realm_name: realm.name.clone(),
                    config: PortalThemeConfig::default(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn update_theme_upserts_when_allowed() {
        let realm = test_realm();
        let user = test_user(&realm);
        let mut new_config = PortalThemeConfig::default();
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

        let mut theme_repo = MockPortalThemeRepository::new();
        let realm_for_repo = realm.clone();
        theme_repo
            .expect_upsert()
            .returning(move |_realm_id, config| {
                let stored = stored_theme(&realm_for_repo, config);
                Box::pin(async move { Ok(stored) })
            });

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .update_theme(
                Identity::User(user),
                UpdateThemeInput {
                    realm_name: realm.name.clone(),
                    config: new_config.clone(),
                },
            )
            .await
            .expect("upsert should succeed");

        assert_eq!(result.config, new_config);
    }

    #[tokio::test]
    async fn get_theme_returns_default_when_repo_empty() {
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

        let mut theme_repo = MockPortalThemeRepository::new();
        theme_repo
            .expect_get_by_realm()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .get_theme(
                Identity::User(user),
                GetThemeInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await
            .expect("get should succeed");

        assert_eq!(result, PortalThemeConfig::default());
    }

    #[tokio::test]
    async fn get_theme_returns_stored_config() {
        let realm = test_realm();
        let user = test_user(&realm);
        let mut stored_config = PortalThemeConfig::default();
        stored_config.borders.button_radius = 12;

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

        let mut theme_repo = MockPortalThemeRepository::new();
        let realm_for_repo = realm.clone();
        let stored_clone = stored_config.clone();
        theme_repo.expect_get_by_realm().returning(move |_| {
            let stored = stored_theme(&realm_for_repo, stored_clone.clone());
            Box::pin(async move { Ok(Some(stored)) })
        });

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .get_theme(
                Identity::User(user),
                GetThemeInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await
            .expect("get should succeed");

        assert_eq!(result, stored_config);
    }

    #[tokio::test]
    async fn get_public_theme_skips_policy_and_returns_default_when_empty() {
        let realm = test_realm();

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();

        let mut theme_repo = MockPortalThemeRepository::new();
        theme_repo
            .expect_get_by_realm()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .get_public_theme(GetThemeInput {
                realm_name: realm.name.clone(),
            })
            .await
            .expect("public get should succeed");

        assert_eq!(result, PortalThemeConfig::default());
    }

    #[tokio::test]
    async fn get_public_theme_returns_stored_config_without_auth() {
        let realm = test_realm();
        let mut stored_config = PortalThemeConfig::default();
        stored_config.colors.primary_button = "#abcdef".to_string();

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();

        let mut theme_repo = MockPortalThemeRepository::new();
        let realm_for_repo = realm.clone();
        let stored_clone = stored_config.clone();
        theme_repo.expect_get_by_realm().returning(move |_| {
            let stored = stored_theme(&realm_for_repo, stored_clone.clone());
            Box::pin(async move { Ok(Some(stored)) })
        });

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .get_public_theme(GetThemeInput {
                realm_name: realm.name.clone(),
            })
            .await
            .expect("public get should succeed");

        assert_eq!(result, stored_config);
    }
}
