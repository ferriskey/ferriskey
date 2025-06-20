use crate::{
    application::{
        auth::Identity,
        http::{
            policies::PolicyEnforcer,
            server::{api_entities::api_error::ApiError, app_state::AppState},
        },
    },
    domain::{realm::entities::realm::Realm, role::entities::permission::Permissions},
};

/// Policy for managing user role access control
///
/// This policy determines whether a user can view user roles based on their permissions.
/// The logic follows a hierarchical approach:
/// 1. Users with realm management permissions have full access
/// 2. Users need both role viewing and user viewing permissions for restricted access
pub struct UserRolePolicy;

impl UserRolePolicy {
    /// Check if the user can view user roles in the target realm
    ///
    /// # Arguments
    /// * `identity` - The authenticated user's identity
    /// * `state` - Application state containing services
    /// * `target_realm` - The realm where the user roles are being accessed
    ///
    /// # Returns
    /// * `Ok(true)` - User has permission to view user roles
    /// * `Ok(false)` - User does not have sufficient permissions
    /// * `Err(ApiError)` - Error occurred during permission check
    pub async fn view(
        identity: Identity,
        state: AppState,
        target_realm: Realm,
    ) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state.clone());
        let user = policy.get_user_from_identity(&identity).await?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        // Convert HashSet to Vec only once for better performance
        let permissions_vec: Vec<Permissions> = permissions.iter().cloned().collect();

        // Check for full realm management permissions first (highest priority)
        if Self::has_realm_management_permissions(&permissions_vec) {
            return Ok(true);
        }

        // Check for specific role and user viewing permissions
        Ok(Self::has_role_and_user_viewing_permissions(
            &permissions_vec,
        ))
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
    use super::*;
    use crate::domain::role::entities::permission::Permissions;

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
