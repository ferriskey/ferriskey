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

pub struct UserPolicy;

impl UserPolicy {
    /// Check if the user can delete a user in the target realm
    ///
    /// # Arguments
    /// * `identity` - The authenticated user's identity
    /// * `state` - Application state containing services
    /// * `target_realm` - The realm where the user is being deleted
    ///
    /// # Returns
    /// * `Ok(true)` - User has permission to delete users
    /// * `Ok(false)` - User does not have sufficient permissions
    /// * `Err(ApiError)` - Error occurred during permission check
    pub async fn delete(
        identity: Identity,
        state: AppState,
        target_realm: Realm,
    ) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state.clone());
        let user = policy.get_user_from_identity(&identity).await?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

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
