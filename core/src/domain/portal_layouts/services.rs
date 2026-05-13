use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    portal_layouts::{
        entities::PortalLayout,
        ports::{
            CreateLayoutInput, GetLayoutInput, ListLayoutsInput, PortalLayoutsPolicy,
            PortalLayoutsRepository, PortalLayoutsService, UpdateLayoutInput,
        },
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct PortalLayoutsServiceImpl<R, U, C, UR, PL>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PL: PortalLayoutsRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) layouts_repository: Arc<PL>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, PL> PortalLayoutsServiceImpl<R, U, C, UR, PL>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PL: PortalLayoutsRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        layouts_repository: Arc<PL>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            layouts_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, PL> PortalLayoutsService for PortalLayoutsServiceImpl<R, U, C, UR, PL>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    PL: PortalLayoutsRepository,
{
    async fn list_layouts(
        &self,
        identity: Identity,
        input: ListLayoutsInput,
    ) -> Result<Vec<PortalLayout>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.layouts_repository.list_by_realm(realm.id.into()).await
    }

    async fn get_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.layouts_repository
            .get_by_id(realm.id.into(), input.layout_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn create_layout(
        &self,
        identity: Identity,
        input: CreateLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let realm_uuid: uuid::Uuid = realm.id.into();
        let existing_default = self.layouts_repository.get_default(realm_uuid).await?;
        let make_default = existing_default.is_none();

        self.layouts_repository
            .create(realm_uuid, input.name, input.tree, make_default)
            .await
    }

    async fn update_layout(
        &self,
        identity: Identity,
        input: UpdateLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.layouts_repository
            .update(realm.id.into(), input.layout_id, input.name, input.tree)
            .await
    }

    async fn set_default_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.layouts_repository
            .set_default(realm.id.into(), input.layout_id)
            .await
    }

    async fn delete_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_layouts(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.layouts_repository
            .delete(realm.id.into(), input.layout_id)
            .await
    }

    async fn get_public_default_layout(
        &self,
        input: ListLayoutsInput,
    ) -> Result<Option<PortalLayout>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.layouts_repository.get_default(realm.id.into()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        portal_layouts::ports::MockPortalLayoutsRepository,
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

    fn stored_layout(realm: &Realm, name: &str, is_default: bool) -> PortalLayout {
        PortalLayout {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            name: name.to_string(),
            tree: serde_json::json!([]),
            is_default,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    type ServiceUnderTest = PortalLayoutsServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockPortalLayoutsRepository,
    >;

    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        layouts_repo: MockPortalLayoutsRepository,
    ) -> ServiceUnderTest {
        let client_repo = MockClientRepository::new();
        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));
        PortalLayoutsServiceImpl::new(Arc::new(realm_repo), Arc::new(layouts_repo), policy)
    }

    fn realm_repo_returning(realm: Realm) -> MockRealmRepository {
        let mut repo = MockRealmRepository::new();
        repo.expect_get_by_name().returning(move |_| {
            let r = realm.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        repo
    }

    fn user_repo_returning(user: User) -> MockUserRepository {
        let mut repo = MockUserRepository::new();
        repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });
        repo
    }

    fn user_role_repo_returning(roles: Vec<Role>) -> MockUserRoleRepository {
        let mut repo = MockUserRoleRepository::new();
        repo.expect_get_user_roles().returning(move |_| {
            let r = roles.clone();
            Box::pin(async move { Ok(r) })
        });
        repo
    }

    #[tokio::test]
    async fn list_layouts_forbidden_when_policy_denies() {
        let realm = test_realm();
        let user = test_user(&realm);

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![empty_role(&realm)]),
            MockPortalLayoutsRepository::new(),
        );

        let result = service
            .list_layouts(
                Identity::User(user),
                ListLayoutsInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn list_layouts_returns_repo_results_when_allowed() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        let realm_for_repo = realm.clone();
        layouts_repo.expect_list_by_realm().returning(move |_| {
            let layouts = vec![
                stored_layout(&realm_for_repo, "Layout A", true),
                stored_layout(&realm_for_repo, "Layout B", false),
            ];
            Box::pin(async move { Ok(layouts) })
        });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .list_layouts(
                Identity::User(user),
                ListLayoutsInput {
                    realm_name: realm.name.clone(),
                },
            )
            .await
            .expect("list should succeed");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Layout A");
    }

    #[tokio::test]
    async fn get_layout_returns_not_found_when_repo_returns_none() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        layouts_repo
            .expect_get_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .get_layout(
                Identity::User(user),
                GetLayoutInput {
                    realm_name: realm.name.clone(),
                    layout_id: Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn create_layout_marks_first_layout_as_default() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        layouts_repo
            .expect_get_default()
            .returning(|_| Box::pin(async { Ok(None) }));
        let realm_for_repo = realm.clone();
        layouts_repo
            .expect_create()
            .withf(|_realm_id, _name, _tree, is_default| *is_default)
            .returning(move |_realm_id, name, tree, is_default| {
                let layout = PortalLayout {
                    id: Uuid::new_v4(),
                    realm_id: realm_for_repo.id,
                    name,
                    tree,
                    is_default,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(layout) })
            });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .create_layout(
                Identity::User(user),
                CreateLayoutInput {
                    realm_name: realm.name.clone(),
                    name: "First".to_string(),
                    tree: serde_json::json!([]),
                },
            )
            .await
            .expect("create should succeed");

        assert!(result.is_default);
    }

    #[tokio::test]
    async fn create_layout_does_not_default_when_one_exists() {
        let realm = test_realm();
        let user = test_user(&realm);
        let existing = stored_layout(&realm, "Existing", true);

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        let existing_clone = existing.clone();
        layouts_repo.expect_get_default().returning(move |_| {
            let l = existing_clone.clone();
            Box::pin(async move { Ok(Some(l)) })
        });
        let realm_for_repo = realm.clone();
        layouts_repo
            .expect_create()
            .withf(|_realm_id, _name, _tree, is_default| !(*is_default))
            .returning(move |_realm_id, name, tree, is_default| {
                let layout = PortalLayout {
                    id: Uuid::new_v4(),
                    realm_id: realm_for_repo.id,
                    name,
                    tree,
                    is_default,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(layout) })
            });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .create_layout(
                Identity::User(user),
                CreateLayoutInput {
                    realm_name: realm.name.clone(),
                    name: "Second".to_string(),
                    tree: serde_json::json!([]),
                },
            )
            .await
            .expect("create should succeed");

        assert!(!result.is_default);
    }

    #[tokio::test]
    async fn update_layout_forwards_to_repo() {
        let realm = test_realm();
        let user = test_user(&realm);
        let layout_id = Uuid::new_v4();

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        let realm_for_repo = realm.clone();
        layouts_repo
            .expect_update()
            .returning(move |_realm_id, id, name, tree| {
                let layout = PortalLayout {
                    id,
                    realm_id: realm_for_repo.id,
                    name,
                    tree,
                    is_default: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(layout) })
            });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .update_layout(
                Identity::User(user),
                UpdateLayoutInput {
                    realm_name: realm.name.clone(),
                    layout_id,
                    name: "Renamed".to_string(),
                    tree: serde_json::json!([{ "id": "x" }]),
                },
            )
            .await
            .expect("update should succeed");

        assert_eq!(result.id, layout_id);
        assert_eq!(result.name, "Renamed");
    }

    #[tokio::test]
    async fn delete_layout_forbidden_for_non_admin() {
        let realm = test_realm();
        let user = test_user(&realm);

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![empty_role(&realm)]),
            MockPortalLayoutsRepository::new(),
        );

        let result = service
            .delete_layout(
                Identity::User(user),
                GetLayoutInput {
                    realm_name: realm.name.clone(),
                    layout_id: Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn set_default_forwards_to_repo() {
        let realm = test_realm();
        let user = test_user(&realm);
        let layout_id = Uuid::new_v4();

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        let realm_for_repo = realm.clone();
        layouts_repo
            .expect_set_default()
            .returning(move |_realm_id, id| {
                let layout = PortalLayout {
                    id,
                    realm_id: realm_for_repo.id,
                    name: "L".to_string(),
                    tree: serde_json::json!([]),
                    is_default: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(layout) })
            });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            user_repo_returning(user.clone()),
            user_role_repo_returning(vec![admin_role(&realm)]),
            layouts_repo,
        );

        let result = service
            .set_default_layout(
                Identity::User(user),
                GetLayoutInput {
                    realm_name: realm.name.clone(),
                    layout_id,
                },
            )
            .await
            .expect("set_default should succeed");

        assert!(result.is_default);
        assert_eq!(result.id, layout_id);
    }

    #[tokio::test]
    async fn get_public_default_layout_skips_policy() {
        let realm = test_realm();
        let stored = stored_layout(&realm, "Public", true);

        let mut layouts_repo = MockPortalLayoutsRepository::new();
        let stored_clone = stored.clone();
        layouts_repo.expect_get_default().returning(move |_| {
            let l = stored_clone.clone();
            Box::pin(async move { Ok(Some(l)) })
        });

        let service = build_service(
            realm_repo_returning(realm.clone()),
            MockUserRepository::new(),
            MockUserRoleRepository::new(),
            layouts_repo,
        );

        let result = service
            .get_public_default_layout(ListLayoutsInput {
                realm_name: realm.name.clone(),
            })
            .await
            .expect("public default fetch should succeed");

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Public");
    }
}
