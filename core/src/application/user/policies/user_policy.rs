use crate::{
    application::common::policies::PolicyEnforcer,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
        realm::entities::Realm,
        role::entities::permission::Permissions,
        user::{entities::UserError, ports::UserService},
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
    pub async fn delete<U, C>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, UserError>
    where
        U: UserService,
        C: ClientService,
    {
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

    /// Check if the user can store users in the target realm
    ///
    /// # Arguments
    /// * `identity` - The identity of the user
    /// * `target_realm` - The realm in which the user wants to store users
    /// * `user_service` - The service used to manage users
    /// * `client_service` - The service used to manage clients
    ///
    /// # Returns
    /// * `Ok(true)` if the user can store users in the target realm
    /// * `Ok(false)` if the user cannot store users in the target realm
    /// * `Err(UserError)` if an error occurs
    pub async fn store<U, C>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, UserError>
    where
        U: UserService,
        C: ClientService,
    {
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

    /// Check if the user can update users in the target realm
    ///
    /// # Arguments
    /// * `identity` - The identity of the user
    /// * `target_realm` - The realm to check permissions for
    /// * `user_service` - The user service to use for user management
    /// * `client_service` - The client service to use for client management
    ///
    /// # Returns
    /// * `Ok(true)` if the user can update users in the target realm
    /// * `Ok(false)` if the user cannot update users in the target realm
    /// * `Err(UserError)` if an error occurred while checking permissions
    pub async fn update<U, C>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, UserError>
    where
        U: UserService,
        C: ClientService,
    {
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

    pub async fn view<U, C>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, UserError>
    where
        U: UserService,
        C: ClientService,
    {
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

        let can_view =
            Permissions::has_one_of_permissions(&permissions_vec, &[Permissions::ViewUsers]);

        Ok(Self::has_user_management_permissions(&permissions_vec) || can_view)
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
