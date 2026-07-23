//! Tests for the `IdentityProviderPolicy` impl on `FerriskeyPolicy`.
//! The impl now lives in `ferriskey-abyss`; these tests stay in `core` where the repository
//! mocks (`mock` feature) and cross-domain entities are already wired up.
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::domain::abyss::identity_provider::{
        IdentityProvider, IdentityProviderConfig, IdentityProviderCreationConfig,
        IdentityProviderPolicy,
    };
    use crate::domain::authentication::value_objects::Identity;
    use crate::domain::client::ports::MockClientRepository;
    use crate::domain::common::policies::FerriskeyPolicy;
    use crate::domain::realm::entities::{Realm, RealmId};
    use crate::domain::role::entities::Role;
    use crate::domain::role::entities::permission::Permissions;
    use crate::domain::user::entities::User;
    use crate::domain::user::ports::{MockUserRepository, MockUserRoleRepository};

    fn create_test_realm(name: &str) -> Realm {
        Realm {
            id: RealmId::default(),
            name: name.to_string(),
            display_name: None,
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
            email: Some("test@example.com".to_string()),
            email_verified: true,
            firstname: Some("Test".to_string()),
            lastname: Some("User".to_string()),
            enabled: true,
            roles: Some(vec![]),
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
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
            require_mfa: false,
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
