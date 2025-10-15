use std::{collections::HashSet, sync::Arc};

use crate::domain::{
    authentication::value_objects::Identity,
    client::{entities::Client, ports::ClientRepository},
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::entities::{Role, permission::Permissions},
    user::{
        entities::User,
        ports::{UserRepository, UserRoleRepository},
    },
};

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

#[derive(Clone)]
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
    fn has_user_management_permissions(permissions: &[Permissions]) -> bool {
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
