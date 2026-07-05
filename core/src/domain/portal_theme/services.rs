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
            CreateThemeInput, GetThemeByIdInput, GetThemeInput, ListThemesInput, PortalThemePolicy,
            PortalThemeRepository, PortalThemeService, UpdateThemeInput, UpdateThemeMetadataInput,
            UpdateThemePageInput,
        },
        validation::{validate_pages, validate_tree},
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

    async fn list_themes(
        &self,
        identity: Identity,
        input: ListThemesInput,
    ) -> Result<Vec<PortalTheme>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_theme_repository
            .list_by_realm(realm.id.into())
            .await
    }

    async fn get_theme_by_id(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<PortalTheme, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_theme_repository
            .get_by_id(realm.id.into(), input.theme_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn create_theme(
        &self,
        identity: Identity,
        input: CreateThemeInput,
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
            .create(realm.id.into(), input.name, input.layout_id, input.config)
            .await
    }

    async fn update_theme_metadata(
        &self,
        identity: Identity,
        input: UpdateThemeMetadataInput,
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
            .update_metadata(
                realm.id.into(),
                input.theme_id,
                input.name,
                input.layout_id,
                input.config,
            )
            .await
    }

    async fn update_theme_page(
        &self,
        identity: Identity,
        input: UpdateThemePageInput,
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

        // Required-block validation only applies to the currently active
        // theme: a draft can be saved with any tree (validation runs at
        // activation), but the live theme must always satisfy requirements
        // because the public portal already renders it.
        let active_theme = self
            .portal_theme_repository
            .get_active(realm.id.into())
            .await?;
        if active_theme
            .as_ref()
            .map(|t| t.id == input.theme_id)
            .unwrap_or(false)
        {
            validate_tree(input.page_type, &input.tree).map_err(|missing| {
                CoreError::PortalThemePageInvalid(
                    serde_json::to_string(&missing).unwrap_or_else(|_| "{}".to_string()),
                )
            })?;
        }

        self.portal_theme_repository
            .update_page(realm.id.into(), input.theme_id, input.page_type, input.tree)
            .await
    }

    async fn activate_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        // Validate every page tree before flipping the active flag: once a
        // theme goes live the public portal renders it immediately, so a
        // missing required block would break authentication.
        let theme = self
            .portal_theme_repository
            .get_by_id(realm.id.into(), input.theme_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        validate_pages(|pt| theme.pages.get(pt).clone()).map_err(|missing| {
            CoreError::PortalThemeInvalidForActivation(
                serde_json::to_string(&missing).unwrap_or_else(|_| "[]".to_string()),
            )
        })?;

        self.portal_theme_repository
            .activate(realm.id.into(), input.theme_id)
            .await
    }

    async fn delete_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_theme(&identity, &realm).await,
            "insufficient permissions",
        )?;

        if let Some(active) = self
            .portal_theme_repository
            .get_active(realm.id.into())
            .await?
            && active.id == input.theme_id
        {
            return Err(CoreError::PortalThemeActive);
        }

        self.portal_theme_repository
            .delete(realm.id.into(), input.theme_id)
            .await
    }

    async fn get_active_theme(
        &self,
        input: ListThemesInput,
    ) -> Result<Option<PortalTheme>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.portal_theme_repository
            .get_active(realm.id.into())
            .await
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
            display_name: None,
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
            name: "Default".to_string(),
            layout_id: None,
            config,
            pages: Default::default(),
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

    fn build_active_theme_repo(realm: &Realm, active_theme_id: Uuid) -> MockPortalThemeRepository {
        use crate::domain::portal_theme::entities::PortalThemePages;

        let mut theme_repo = MockPortalThemeRepository::new();
        let realm_for_active = realm.clone();
        theme_repo.expect_get_active().returning(move |_| {
            let theme = PortalTheme {
                id: active_theme_id,
                realm_id: realm_for_active.id,
                name: "Active".to_string(),
                layout_id: None,
                config: PortalThemeConfig::default(),
                pages: PortalThemePages::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            Box::pin(async move { Ok(Some(theme)) })
        });
        theme_repo
    }

    #[tokio::test]
    async fn update_theme_page_accepts_invalid_tree_for_inactive_theme() {
        use crate::domain::portal_theme::entities::{PortalPageType, PortalThemePages};
        use serde_json::json;

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

        // Draft theme — the active theme has a *different* id, so validation
        // is skipped and the invalid tree gets persisted.
        let mut theme_repo = MockPortalThemeRepository::new();
        theme_repo
            .expect_get_active()
            .returning(|_| Box::pin(async { Ok(None) }));
        let realm_for_update = realm.clone();
        theme_repo
            .expect_update_page()
            .returning(move |_, theme_id, _, tree| {
                let theme = PortalTheme {
                    id: theme_id,
                    realm_id: realm_for_update.id,
                    name: "Draft".to_string(),
                    layout_id: None,
                    config: PortalThemeConfig::default(),
                    pages: PortalThemePages {
                        login: tree,
                        ..Default::default()
                    },
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(theme) })
            });

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .update_theme_page(
                Identity::User(user),
                UpdateThemePageInput {
                    realm_name: realm.name.clone(),
                    theme_id: Uuid::new_v4(),
                    page_type: PortalPageType::Login,
                    tree: json!([{ "type": "email_input" }]),
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_theme_page_rejects_invalid_tree_when_theme_is_active() {
        use crate::domain::portal_theme::entities::PortalPageType;
        use serde_json::json;

        let realm = test_realm();
        let user = test_user(&realm);
        let active_theme_id = Uuid::new_v4();

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

        let theme_repo = build_active_theme_repo(&realm, active_theme_id);
        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .update_theme_page(
                Identity::User(user),
                UpdateThemePageInput {
                    realm_name: realm.name.clone(),
                    theme_id: active_theme_id,
                    page_type: PortalPageType::Login,
                    tree: json!([{ "type": "email_input" }]),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::PortalThemePageInvalid(_))));
    }

    #[tokio::test]
    async fn activate_theme_rejects_when_any_page_is_missing_required_blocks() {
        let realm = test_realm();
        let user = test_user(&realm);
        let theme_id = Uuid::new_v4();

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

        // Theme has default (empty) pages: every required block is missing.
        let mut theme_repo = MockPortalThemeRepository::new();
        let realm_for_get = realm.clone();
        theme_repo.expect_get_by_id().returning(move |_, id| {
            let theme = PortalTheme {
                id,
                realm_id: realm_for_get.id,
                name: "Draft".to_string(),
                layout_id: None,
                config: PortalThemeConfig::default(),
                pages: Default::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            Box::pin(async move { Ok(Some(theme)) })
        });

        let service = build_service(realm_repo, user_repo, user_role_repo, theme_repo);

        let result = service
            .activate_theme(
                Identity::User(user),
                GetThemeByIdInput {
                    realm_name: realm.name.clone(),
                    theme_id,
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(CoreError::PortalThemeInvalidForActivation(_))
        ));
    }
}
