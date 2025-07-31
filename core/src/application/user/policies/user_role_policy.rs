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

/// Policy for managing user role access control
///
/// This policy determines whether a user can view user roles based on their permissions.
/// The logic follows a hierarchical approach:
/// 1. Users with realm management permissions have full access
/// 2. Users need both role viewing and user viewing permissions for restricted access
pub struct UserRolePolicy;

impl UserRolePolicy {
    /// Chekc if the user can view user roles in the target realm
    ///
    /// # Arguments
    /// * `identity` - The identity of the user making the request.
    /// * `target_realm` - The realm for which the user is requesting role information.
    /// * `user_service` - The service for managing user data.
    /// * `client_service` - The service for managing client data.
    ///
    /// # Returns
    /// * `Ok(true)` if the user can view user roles in the target realm.
    /// * `Ok(false)` if the user cannot view user roles in the target realm.
    /// * `Err(UserError)` if an error occurs while processing the request.
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

        if Self::has_realm_management_permissions(&permissions_vec) {
            return Ok(true);
        }

        Ok(Self::has_role_and_user_viewing_permissions(
            &permissions_vec,
        ))
    }

    /// Check if the user can store user roles in the target realm
    ///
    /// # Arguments
    /// * `identity` - The authenticated user's identity
    /// * `target_realm` - The realm where the user roles are being stored
    /// * `user_service` - The user service to use for retrieving the user
    /// * `client_service` - The client service to use for retrieving the client
    ///
    /// # Returns
    /// * `Ok(true)` - User has permission to store user roles
    /// * `Ok(false)` - User does not have sufficient permissions
    /// * `Err(UserError)` - Error occurred during permission check
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

        if Self::has_realm_management_permissions(&permissions_vec) {
            return Ok(true);
        }

        Ok(Self::has_role_and_user_management_permissions(
            &permissions_vec,
        ))
    }

    /// Check if the user can delete user roles in the target realm
    ///
    /// # Arguments
    /// * `identity` - The authenticated user's identity
    /// * `target_realm` - The realm where the user roles are being deleted
    /// * `user_service` - Service for managing users
    /// * `client_service` - Service for managing clients
    ///
    /// # Returns
    /// * `Ok(true)` - User has permission to delete user roles
    /// * `Ok(false)` - User does not have sufficient permissions
    /// * `Err(UserError)` - Error occurred during permission check
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

        if Self::has_realm_management_permissions(&permissions_vec) {
            return Ok(true);
        }

        Ok(Self::has_role_and_user_management_permissions(
            &permissions_vec,
        ))
    }

    /// Check if user has both role management and user management permissions
    ///
    /// Users with these permissions can manage user roles and users in a restricted manner
    #[inline]
    fn has_role_and_user_management_permissions(permissions: &[Permissions]) -> bool {
        Permissions::has_permissions(
            permissions,
            &[Permissions::ManageUsers, Permissions::ManageRoles],
        )
    }

    /// Check if user has realm management permissions
    ///
    /// Users with these permissions have full access to view user roles
    #[inline]
    fn has_realm_management_permissions(permissions: &[Permissions]) -> bool {
        Permissions::has_one_of_permissions(
            permissions,
            &[Permissions::ManageRealm, Permissions::ManageRoles],
        )
    }

    /// Check if user has both role viewing and user viewing permissions
    ///
    /// Users need both permissions to view user roles in a restricted manner
    #[inline]
    fn has_role_and_user_viewing_permissions(permissions: &[Permissions]) -> bool {
        let can_view_roles = Permissions::has_one_of_permissions(
            permissions,
            &[Permissions::ViewRoles, Permissions::ManageRoles],
        );

        let can_view_users = Permissions::has_one_of_permissions(
            permissions,
            &[Permissions::ViewUsers, Permissions::ManageUsers],
        );

        can_view_roles && can_view_users
    }
}

#[cfg(test)]
mod tests {
    use crate::application::user::policies::user_role_policy::UserRolePolicy;
    use crate::domain::{
        role::entities::permission::Permissions,
    };

    #[test]
    fn test_has_realm_management_permissions() {
        let permissions = vec![Permissions::ManageRealm];
        assert!(UserRolePolicy::has_realm_management_permissions(
            &permissions
        ));

        let permissions = vec![Permissions::ManageRoles];
        assert!(UserRolePolicy::has_realm_management_permissions(
            &permissions
        ));

        let permissions = vec![Permissions::ViewRoles, Permissions::ViewUsers];
        assert!(!UserRolePolicy::has_realm_management_permissions(
            &permissions
        ));
    }

    #[test]
    fn test_has_role_and_user_viewing_permissions() {
        let permissions = vec![Permissions::ViewRoles, Permissions::ViewUsers];
        assert!(UserRolePolicy::has_role_and_user_viewing_permissions(
            &permissions
        ));

        let permissions = vec![Permissions::ManageRoles, Permissions::ManageUsers];
        assert!(UserRolePolicy::has_role_and_user_viewing_permissions(
            &permissions
        ));

        let permissions = vec![Permissions::ViewRoles];
        assert!(!UserRolePolicy::has_role_and_user_viewing_permissions(
            &permissions
        ));

        let permissions = vec![Permissions::ViewUsers];
        assert!(!UserRolePolicy::has_role_and_user_viewing_permissions(
            &permissions
        ));
    }
}
