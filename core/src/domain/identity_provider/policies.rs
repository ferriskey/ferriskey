use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    identity_provider::{entities::IdentityProvider, ports::IdentityProviderPolicy},
    realm::entities::{Realm, RealmId},
    role::entities::permission::Permissions,
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> IdentityProviderPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    /// Checks if the identity can create identity providers in the given realm
    ///
    /// Requires ManageRealm permission on the target realm.
    async fn can_create_identity_provider(
        &self,
        identity: Identity,
        realm_id: RealmId,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        // Get the user's realm to check access
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: realm_id,
            name: user_realm.name.clone(),
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(has_permission)
    }

    /// Checks if the identity can view the given identity provider
    ///
    /// Requires ViewRealm or ManageRealm permission on the provider's realm.
    async fn can_view_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == provider.realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: provider.realm_id,
            name: user_realm.name.clone(),
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ViewRealm],
        );

        Ok(has_permission)
    }

    /// Checks if the identity can update the given identity provider
    ///
    /// Requires ManageRealm permission on the provider's realm.
    async fn can_update_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == provider.realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: provider.realm_id,
            name: user_realm.name.clone(),
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(has_permission)
    }

    /// Checks if the identity can delete the given identity provider
    ///
    /// Requires ManageRealm permission on the provider's realm.
    async fn can_delete_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        // Delete has the same requirements as update
        self.can_update_identity_provider(identity, provider).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        identity_provider::entities::{IdentityProviderConfig, IdentityProviderCreationConfig},
        realm::entities::Realm,
        role::entities::Role,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRoleRepository},
        },
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_test_realm(name: &str) -> Realm {
        Realm {
            id: RealmId::default(),
            name: name.to_string(),
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_user_with_realm(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            enabled: true,
            roles: vec![],
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_identity_provider(realm_id: RealmId) -> IdentityProvider {
        IdentityProvider::new(IdentityProviderCreationConfig {
            realm_id,
            alias: "google".to_string(),
            provider_id: "oidc".to_string(),
            enabled: true,
            display_name: Some("Google".to_string()),
            first_broker_login_flow_alias: None,
            post_broker_login_flow_alias: None,
            store_token: false,
            add_read_token_role_on_create: false,
            trust_email: false,
            link_only: false,
            config: IdentityProviderConfig {
                client_id: None,
                client_secret: None,
                extra: serde_json::json!({}),
            },
        })
    }

    fn create_role_with_permission(realm_id: RealmId, permission: Permissions) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "test_role".to_string(),
            description: None,
            permissions: vec![permission.name()],
            realm_id,
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_can_view_identity_provider_with_manage_realm_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
        let provider = create_test_identity_provider(realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let mut user_role_repo = MockUserRoleRepository::new();

        let role = create_role_with_permission(realm.id, Permissions::ManageRealm);
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_view_identity_provider_with_view_realm_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
        let provider = create_test_identity_provider(realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let mut user_role_repo = MockUserRoleRepository::new();

        let role = create_role_with_permission(realm.id, Permissions::ViewRealm);
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_view_identity_provider_without_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
        let provider = create_test_identity_provider(realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let mut user_role_repo = MockUserRoleRepository::new();

        user_role_repo
            .expect_get_user_roles()
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_can_update_identity_provider_requires_manage_realm() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
        let provider = create_test_identity_provider(realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let mut user_role_repo = MockUserRoleRepository::new();

        let role = create_role_with_permission(realm.id, Permissions::ManageRealm);
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_update_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_update_identity_provider_with_only_view_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
        let provider = create_test_identity_provider(realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let mut user_role_repo = MockUserRoleRepository::new();

        let role = create_role_with_permission(realm.id, Permissions::ViewRealm);
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_update_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_access_identity_provider_from_different_realm() {
        let user_realm = create_test_realm("user_realm");
        let provider_realm = create_test_realm("provider_realm");
        let user = create_test_user_with_realm(&user_realm);
        let provider = create_test_identity_provider(provider_realm.id);
        let identity = Identity::User(user.clone());

        let user_repo = MockUserRepository::new();
        let client_repo = MockClientRepository::new();
        let user_role_repo = MockUserRoleRepository::new();

        let policy = FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &provider)
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
