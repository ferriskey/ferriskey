use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    portal::{
        entities::PortalConfig,
        ports::{
            DeletePortalConfigInput, DisablePortalConfigInput, EnablePortalConfigInput,
            GetPortalConfigInput, PortalPolicy, PortalRepository, PortalService,
            UpsertPortalConfigInput,
        },
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct PortalServiceImpl<R, U, C, UR, P>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    P: PortalRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) portal_repository: Arc<P>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, P> PortalServiceImpl<R, U, C, UR, P>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    P: PortalRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        portal_repository: Arc<P>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            portal_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, P> PortalService for PortalServiceImpl<R, U, C, UR, P>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    P: PortalRepository,
{
    async fn get_portal_config(
        &self,
        identity: Identity,
        input: GetPortalConfigInput,
    ) -> Result<Option<PortalConfig>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_portal(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_repository.get_by_realm(realm.id.into()).await
    }

    async fn upsert_portal_config(
        &self,
        identity: Identity,
        input: UpsertPortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_portal(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_repository
            .upsert(realm.id.into(), input.layout)
            .await
    }

    async fn delete_portal_config(
        &self,
        identity: Identity,
        input: DeletePortalConfigInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_portal(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_repository
            .get_by_realm(realm.id.into())
            .await?
            .ok_or(CoreError::PortalConfigNotFound)?;

        self.portal_repository.delete(realm.id.into()).await
    }

    async fn enable_portal_config(
        &self,
        identity: Identity,
        input: EnablePortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_portal(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_repository
            .get_by_realm(realm.id.into())
            .await?
            .ok_or(CoreError::PortalConfigNotFound)?;

        self.portal_repository
            .set_active(realm.id.into(), true)
            .await
    }

    async fn disable_portal_config(
        &self,
        identity: Identity,
        input: DisablePortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_manage_portal(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.portal_repository
            .get_by_realm(realm.id.into())
            .await?
            .ok_or(CoreError::PortalConfigNotFound)?;

        self.portal_repository
            .set_active(realm.id.into(), false)
            .await
    }

    async fn get_active_portal_config(
        &self,
        realm_name: &str,
    ) -> Result<Option<PortalConfig>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.portal_repository
            .get_active_by_realm(realm.id.into())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        portal::ports::MockPortalRepository,
        realm::{entities::Realm, ports::MockRealmRepository},
        role::entities::Role,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRoleRepository},
        },
    };
    use chrono::Utc;
    use serde_json::json;

    fn test_realm() -> Realm {
        Realm {
            id: uuid::Uuid::new_v4().into(),
            name: "test-realm".to_string(),
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id,
            username: "admin".to_string(),
            firstname: "Admin".to_string(),
            lastname: "User".to_string(),
            email: "admin@test.com".to_string(),
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

    fn test_portal_config(realm: &Realm) -> PortalConfig {
        PortalConfig {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id.into(),
            is_active: false,
            layout: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn admin_role(realm: &Realm) -> Role {
        Role {
            id: uuid::Uuid::new_v4(),
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

    #[tokio::test]
    async fn test_upsert_portal_config() {
        let realm = test_realm();
        let user = test_user(&realm);
        let config = test_portal_config(&realm);
        let realm_clone = realm.clone();
        let config_clone = config.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let role_realm_id = realm.id;
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let rid = role_realm_id;
            Box::pin(async move {
                Ok(vec![Role {
                    id: uuid::Uuid::new_v4(),
                    name: "admin".to_string(),
                    description: None,
                    permissions: vec!["manage_realm".to_string()],
                    realm_id: rid,
                    client_id: None,
                    client: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }])
            })
        });

        let client_repo = MockClientRepository::new();

        let mut portal_repo = MockPortalRepository::new();
        portal_repo.expect_upsert().returning(move |_, _| {
            let c = config_clone.clone();
            Box::pin(async move { Ok(c) })
        });

        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));

        let service = PortalServiceImpl::new(Arc::new(realm_repo), Arc::new(portal_repo), policy);

        let identity = Identity::User(test_user(&realm));

        let result = service
            .upsert_portal_config(
                identity,
                UpsertPortalConfigInput {
                    realm_name: "test-realm".to_string(),
                    layout: json!({}),
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_portal_config_not_found() {
        let realm = test_realm();
        let user = test_user(&realm);
        let realm_clone = realm.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let role_realm_id = realm.id;
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let rid = role_realm_id;
            Box::pin(async move {
                Ok(vec![admin_role(&Realm {
                    id: rid,
                    name: "test-realm".to_string(),
                    settings: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })])
            })
        });

        let client_repo = MockClientRepository::new();

        let mut portal_repo = MockPortalRepository::new();
        portal_repo
            .expect_get_by_realm()
            .returning(|_| Box::pin(async { Ok(None) }));

        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));

        let service = PortalServiceImpl::new(Arc::new(realm_repo), Arc::new(portal_repo), policy);

        let identity = Identity::User(test_user(&realm));

        let result = service
            .delete_portal_config(
                identity,
                DeletePortalConfigInput {
                    realm_name: "test-realm".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::PortalConfigNotFound)));
    }
}
