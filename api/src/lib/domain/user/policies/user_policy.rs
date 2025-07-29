use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        core::policy::PolicyEnforcer,
        realm::entities::realm::Realm,
        role::entities::permission::Permissions,
        user::{entities::error::UserError, services::user_service::DefaultUserService},
    },
};

pub struct UserPolicy;

impl UserPolicy {
    /// Check if the user can delete a user in the target realm
    ///
    /// # Arguments
    /// * `identity` - Identity of the user making the request
    /// * `target_realm` - Realm where the user is being deleted
    /// * `user_service` - Service for managing users
    /// * `client_service` - Service for managing clients
    /// # Returns
    /// * `Ok(true)` - User has permission to delete the user
    /// * `Ok(false)` - User does not have permission to delete the user
    /// * `Err(UserError)` - Error occurred while checking permissions
    pub async fn delete(
        identity: Identity,
        target_realm: Realm,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Result<bool, UserError> {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let permissions_vec: Vec<Permissions> = permissions.iter().cloned().collect();

        Ok(Self::has_user_management_permissions(&permissions_vec))
    }

    pub async fn store(
        identity: Identity,
        target_realm: Realm,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Result<bool, UserError> {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let permissions_vec: Vec<Permissions> = permissions.iter().cloned().collect();

        Ok(Self::has_user_management_permissions(&permissions_vec))
    }

    /// Check if the user can manage users in the target realm
    ///
    /// # Arguments
    /// * `permissions` - List of permissions the user has
    /// # Returns
    /// * `true` - User has permission to manage users
    /// * `false` - User does not have sufficient permissions
    #[inline]
    fn has_user_management_permissions(permissions: &[Permissions]) -> bool {
        Permissions::has_one_of_permissions(
            permissions,
            &[Permissions::ManageUsers, Permissions::ManageRealm],
        )
    }
}
