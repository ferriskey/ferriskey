use std::collections::HashSet;
use std::sync::Arc;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::{ClientPolicy, ClientRepository};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::Policy;
use ferriskey_domain::realm::Realm;
use ferriskey_domain::realm::ports::RealmPolicy;
use ferriskey_domain::realm::scope::RealmScope;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::role::ports::RolePolicy;
use ferriskey_domain::user::entities::User;
use ferriskey_domain::user::ports::{UserPolicy, UserRepository, UserRoleRepository};

/// The RBAC engine FerrisKey ships today.
///
/// Resolves a principal to a set of [`Permissions`] and answers each
/// `can_*` question by testing that set against a fixed slice. The
/// per-domain `XxxPolicy` traits are implemented on this type from their own
/// crates — a local trait on a foreign type, so the orphan rule allows it.
#[derive(Clone, Debug)]
pub struct FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    user_repository: Arc<U>,
    client_repository: Arc<C>,
    user_role_repository: Arc<UR>,
}

impl<U, C, UR> FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        user_repository: Arc<U>,
        client_repository: Arc<C>,
        user_role_repository: Arc<UR>,
    ) -> Self {
        Self {
            user_repository,
            client_repository,
            user_role_repository,
        }
    }
}

impl<U, C, UR> Policy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, CoreError> {
        match identity {
            Identity::User(user) => Ok(user.clone()),
            Identity::Client(client) => {
                let service_account = self
                    .user_repository
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|e| CoreError::Forbidden(e.to_string()))?;

                Ok(service_account.across_realms())
            }
        }
    }

    async fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> Result<HashSet<Permissions>, CoreError> {
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        if !self.can_access_realm(user_realm, target_realm) {
            return Err(CoreError::NotFound);
        }

        let mut permissions: HashSet<Permissions> = HashSet::new();

        if user_realm.name == "master" {
            let mirror_client_id = format!("{}-realm", target_realm.name);

            let user_scope = RealmScope::from_realm(user_realm.clone());

            let mirror_client = match self
                .client_repository
                .get_by_client_id(mirror_client_id, user_realm.id)
                .await
            {
                Ok(client) => Some(client.in_realm(&user_scope)?),
                Err(CoreError::NotFound) => None,
                Err(error) => return Err(error),
            };

            let targets_own_realm = user_realm.name == target_realm.name;

            let roles = self
                .user_role_repository
                .get_user_roles(user.id)
                .await
                .map_err(|_| CoreError::Forbidden("user not found".to_string()))?;

            for role in roles {
                let role_grants_on_target = match role.client_id {
                    None => targets_own_realm,
                    Some(role_client_id) => mirror_client
                        .as_ref()
                        .is_some_and(|client| client.get().id == role_client_id),
                };

                if role_grants_on_target {
                    let role_permissions: HashSet<Permissions> = role
                        .permissions
                        .iter()
                        .filter_map(|p| Permissions::from_name(p))
                        .collect();

                    let permissions_as_vec: Vec<Permissions> =
                        role_permissions.into_iter().collect();
                    let permissions_bits = Permissions::to_bitfield(&permissions_as_vec);
                    let validated_permissions = Permissions::from_bitfield(permissions_bits);

                    permissions.extend(validated_permissions);
                }
            }
        } else {
            let user_permissions = self.get_user_permissions(user).await?;
            permissions.extend(user_permissions);
        }

        Ok(permissions)
    }

    async fn get_user_permissions(&self, user: &User) -> Result<HashSet<Permissions>, CoreError> {
        let roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::Forbidden("user not found".to_string()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        for role in roles {
            let role_permissions: HashSet<Permissions> = role
                .permissions
                .iter()
                .filter_map(|p| Permissions::from_name(p))
                .collect();

            let permissions_as_vec: Vec<Permissions> = role_permissions.into_iter().collect();
            let permissions_bits = Permissions::to_bitfield(&permissions_as_vec);
            let validated_permissions = Permissions::from_bitfield(permissions_bits);

            permissions.extend(validated_permissions);
        }

        Ok(permissions)
    }

    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        user_realm.name == target_realm.name || user_realm.name == "master"
    }
}

impl<U, C, UR> UserPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_user(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_delete_user(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_update_user(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_view_user(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ViewUsers],
        );

        Ok(has_permission)
    }

    async fn can_view_user_permissions(
        &self,
        identity: &Identity,
        target_realm: &Realm,
        target_user_id: uuid::Uuid,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        if user.id == target_user_id {
            return Ok(true);
        }

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ManageRealm,
                Permissions::ManageUsers,
                Permissions::ViewUsers,
            ],
        );

        Ok(has_permission)
    }
}

impl<U, C, UR> RealmPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_view_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ManageRealm,
                Permissions::ManageRealm,
                Permissions::ViewRealm,
            ],
        );

        Ok(has_permission)
    }

    async fn can_create_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageRealm],
        );

        Ok(has_permission)
    }

    async fn can_update_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageRealm],
        );

        Ok(has_permission)
    }

    async fn can_delete_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageRealm],
        );

        Ok(has_permission)
    }
}

impl<U, C, UR> RolePolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_role(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_delete_role(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_update_role(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_view_role(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ManageRealm,
                Permissions::ManageUsers,
                Permissions::ViewRoles,
            ],
        );

        Ok(has_permission)
    }
}

impl<U, C, UR> ClientPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_delete_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_update_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_view_client(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ViewClients],
        );

        Ok(has_permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ferriskey_domain::client::entities::Client;
    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::RealmId;
    use ferriskey_domain::realm::scope::Unscoped;
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository};
    use uuid::Uuid;

    fn make_realm(name: &str) -> Realm {
        Realm {
            id: RealmId::new(Uuid::new_v4()),
            name: name.to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            client_id: None,
            username: "caller".to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
            locale: None,
        }
    }

    fn make_role(realm_id: RealmId, client_id: Option<Uuid>, permissions: Vec<String>) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "role".to_string(),
            description: None,
            permissions,
            realm_id,
            client_id,
            client: None,
            require_mfa: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_policy(
        user_role_repo: MockUserRoleRepository,
        client_repo: MockClientRepository,
    ) -> FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository> {
        FerriskeyPolicy::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        )
    }

    #[tokio::test]
    async fn master_user_with_only_a_foreign_tenant_mirror_role_gets_no_permission_on_master() {
        let master = make_realm("master");
        let tenant_mirror =
            Client::from_realm_and_client_id(master.id, "tenant-a-realm".to_string());
        let master_mirror = Client::from_realm_and_client_id(master.id, "master-realm".to_string());
        let user = make_user(&master);

        let malicious_role = make_role(
            master.id,
            Some(tenant_mirror.id),
            vec![Permissions::ManageRealm.name()],
        );

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo
            .expect_get_user_roles()
            .times(1)
            .returning(move |_| {
                let role = malicious_role.clone();
                Box::pin(async move { Ok(vec![role]) })
            });

        let mut client_repo = MockClientRepository::new();
        client_repo
            .expect_get_by_client_id()
            .times(0..=1)
            .returning(move |_, _| {
                let client = master_mirror.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });

        let policy = build_policy(user_role_repo, client_repo);

        let permissions = policy
            .get_permission_for_target_realm(&user, &master)
            .await
            .expect("policy lookup should succeed");

        assert!(
            permissions.is_empty(),
            "a master user holding only a foreign tenant mirror role must gain no permission on master, got {permissions:?}"
        );
    }

    #[tokio::test]
    async fn master_user_with_a_tenant_mirror_role_can_manage_that_tenant() {
        let master = make_realm("master");
        let tenant = make_realm("tenant-a");
        let tenant_mirror =
            Client::from_realm_and_client_id(master.id, "tenant-a-realm".to_string());
        let user = make_user(&master);

        let delegated_role = make_role(
            master.id,
            Some(tenant_mirror.id),
            vec![Permissions::ManageRealm.name()],
        );

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo
            .expect_get_user_roles()
            .times(1)
            .returning(move |_| {
                let role = delegated_role.clone();
                Box::pin(async move { Ok(vec![role]) })
            });

        let mut client_repo = MockClientRepository::new();
        client_repo
            .expect_get_by_client_id()
            .times(0..=1)
            .returning(move |_, _| {
                let client = tenant_mirror.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });

        let policy = build_policy(user_role_repo, client_repo);

        let permissions = policy
            .get_permission_for_target_realm(&user, &tenant)
            .await
            .expect("policy lookup should succeed");

        assert!(
            permissions.contains(&Permissions::ManageRealm),
            "a master user holding the tenant-a mirror role must manage tenant-a, got {permissions:?}"
        );
    }

    #[tokio::test]
    async fn bootstrap_master_admin_keeps_manage_realm_on_master() {
        let master = make_realm("master");
        let master_mirror = Client::from_realm_and_client_id(master.id, "master-realm".to_string());
        let user = make_user(&master);

        let admin_role = make_role(
            master.id,
            Some(master_mirror.id),
            vec![Permissions::ManageRealm.name()],
        );

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo
            .expect_get_user_roles()
            .times(1)
            .returning(move |_| {
                let role = admin_role.clone();
                Box::pin(async move { Ok(vec![role]) })
            });

        let mut client_repo = MockClientRepository::new();
        client_repo
            .expect_get_by_client_id()
            .times(0..=1)
            .returning(move |_, _| {
                let client = master_mirror.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });

        let policy = build_policy(user_role_repo, client_repo);

        let permissions = policy
            .get_permission_for_target_realm(&user, &master)
            .await
            .expect("policy lookup should succeed");

        assert!(
            permissions.contains(&Permissions::ManageRealm),
            "the bootstrap admin (role on the master-realm client) must keep ManageRealm on master, got {permissions:?}"
        );
    }
}
