use std::collections::HashSet;
use std::sync::Arc;

use crate::auth::Identity;
use crate::client::entities::Client;
use crate::client::ports::{ClientPolicy, ClientRepository};
use crate::common::app_errors::CoreError;
use crate::realm::Realm;
use crate::realm::ports::RealmPolicy;
use crate::role::entities::Role;
use crate::role::permission::Permissions;
use crate::role::ports::RolePolicy;
use crate::user::entities::User;
use crate::user::ports::{UserPolicy, UserRepository, UserRoleRepository};

pub trait Policy: Send + Sync {
    fn get_user_from_identity(
        &self,
        identity: &Identity,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;
    fn get_user_permissions(
        &self,
        user: &User,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool;
    fn is_cross_realm_access(&self, user_realm: &Realm, target_realm: &Realm) -> bool;
}

pub fn ensure_policy(
    result_has_permission: Result<bool, CoreError>,
    error_message: &str,
) -> Result<(), CoreError> {
    match result_has_permission {
        Ok(true) => Ok(()),
        Ok(false) => Err(CoreError::Forbidden(error_message.to_string())),
        Err(_) => Err(CoreError::Forbidden(error_message.to_string())),
    }
}

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

    /// Check if the user can manage users in the target realm
    ///
    /// # Arguments
    /// * `permissions` - List of permissions the user has
    /// # Returns
    /// * `true` - User has permission to manage users
    /// * `false` - User does not have sufficient permissions
    #[inline]
    #[allow(dead_code)]
    fn has_user_management_permissions(permissions: &HashSet<Permissions>) -> bool {
        Permissions::has_one_of_permissions(
            permissions,
            &[Permissions::ManageUsers, Permissions::ManageRealm],
        )
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

                Ok(service_account)
            }
        }
    }

    async fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> Result<HashSet<Permissions>, CoreError> {
        let roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::Forbidden("user not found".to_string()))?;

        let client_roles = roles
            .into_iter()
            .filter(|role| role.client_id == Some(client.id))
            .collect::<Vec<Role>>();

        let mut permissions: HashSet<Permissions> = HashSet::new();

        for role in client_roles {
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

    async fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> Result<HashSet<Permissions>, CoreError> {
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        if !self.can_access_realm(user_realm, target_realm) {
            return Ok(permissions);
        }

        if self.is_cross_realm_access(user_realm, target_realm) {
            let client_id = format!("{}-realm", target_realm.name);

            let client = self
                .client_repository
                .get_by_client_id(client_id, user_realm.id)
                .await
                .map_err(|_| {
                    CoreError::Forbidden("client not found for target realm".to_string())
                })?;

            let client_permissions = self.get_client_specific_permissions(user, &client).await?;

            permissions.extend(client_permissions);
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

    fn is_cross_realm_access(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        user_realm.name == "master" && user_realm.name != target_realm.name
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
