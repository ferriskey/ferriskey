//! Tests for the `IdentityProviderPolicy` impl on `FerriskeyPolicy`.
//! The impl now lives in `ferriskey-abyss`; these tests stay in `core` where the repository
//! mocks (`mock` feature) and cross-domain entities are already wired up.
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::domain::abyss::identity_provider::IdentityProviderPolicy;
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

    /// The `{realm}-realm` client through which a master user is delegated rights on
    /// another realm. It lives in the *master* realm, which is where
    /// `get_permission_for_target_realm` looks it up.
    fn create_test_client(
        realm_id: RealmId,
        client_id: &str,
    ) -> crate::domain::client::entities::Client {
        use crate::domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy};

        Client {
            id: Uuid::new_v4(),
            client_id: client_id.to_string(),
            secret: None,
            name: client_id.to_string(),
            realm_id,
            enabled: true,
            public_client: false,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            require_pkce: false,
            service_account_enabled: false,
            client_type: ClientType::Confidential,
            protocol: "openid-connect".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_can_view_identity_provider_with_manage_realm_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
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

        let result = policy.can_view_identity_provider(&identity, &realm).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_view_identity_provider_with_view_realm_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
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

        let result = policy.can_view_identity_provider(&identity, &realm).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_view_identity_provider_without_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
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

        let result = policy.can_view_identity_provider(&identity, &realm).await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_can_update_identity_provider_requires_manage_realm() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
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

        let result = policy.can_update_identity_provider(&identity, &realm).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_update_identity_provider_with_only_view_permission() {
        let realm = create_test_realm("test");
        let user = create_test_user_with_realm(&realm);
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

        let result = policy.can_update_identity_provider(&identity, &realm).await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_access_identity_provider_from_different_realm() {
        let user_realm = create_test_realm("user_realm");
        let provider_realm = create_test_realm("provider_realm");
        let user = create_test_user_with_realm(&user_realm);
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
            .can_view_identity_provider(&identity, &provider_realm)
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // ---- FK-006: master reaching another realm ------------------------------
    //
    // These two are the case the previous suite claimed to cover and did not. Its
    // `test_master_realm_can_access_other_realms` built the target realm with
    // `name: user_realm.name.clone()`, which forced `is_cross_realm_access` to false,
    // so `get_client_specific_permissions` was never called and its mock was never
    // consulted. The assertion passed through the unscoped union instead — the very
    // branch the fix removes. Each test below sets `.times(1)` on the client lookup
    // so that regression cannot pass silently again.

    #[tokio::test]
    async fn master_reaching_another_realm_uses_only_that_realms_client_roles() {
        let master_realm = create_test_realm("master");
        let target_realm = create_test_realm("other");
        let user = create_test_user_with_realm(&master_realm);
        let identity = Identity::User(user.clone());

        let target_client = create_test_client(master_realm.id, "other-realm");
        let client_id = target_client.id;

        let mut client_repo = MockClientRepository::new();
        client_repo
            .expect_get_by_client_id()
            .times(1)
            .returning(move |_, _| {
                let c = target_client.clone();
                Box::pin(async move { Ok(c) })
            });

        // Scoped to the `other-realm` client: this is a legitimate delegation.
        let mut role = create_role_with_permission(master_realm.id, Permissions::ManageRealm);
        role.client_id = Some(client_id);

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &target_realm)
            .await;

        assert!(result.is_ok());
        assert!(
            result.unwrap(),
            "a master role scoped to the target realm's client must grant access"
        );
    }

    #[tokio::test]
    async fn master_realm_role_does_not_leak_into_another_realm() {
        let master_realm = create_test_realm("master");
        let target_realm = create_test_realm("other");
        let user = create_test_user_with_realm(&master_realm);
        let identity = Identity::User(user.clone());

        let target_client = create_test_client(master_realm.id, "other-realm");

        let mut client_repo = MockClientRepository::new();
        client_repo
            .expect_get_by_client_id()
            .times(1)
            .returning(move |_, _| {
                let c = target_client.clone();
                Box::pin(async move { Ok(c) })
            });

        // A plain realm role of `master` — `client_id: None`. Under the old code the
        // unscoped union picked it up and granted access to `other`.
        let role = create_role_with_permission(master_realm.id, Permissions::ManageRealm);

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = role.clone();
            Box::pin(async move { Ok(vec![r]) })
        });

        let policy = FerriskeyPolicy::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        );

        let result = policy
            .can_view_identity_provider(&identity, &target_realm)
            .await;

        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "a master role that is not scoped to the target realm's client must not grant access"
        );
    }
}
