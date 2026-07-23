use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::{Realm, RealmId};
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::identity_provider::{IdentityProvider, IdentityProviderPolicy};

impl<U, C, UR> IdentityProviderPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    /// Checks if the identity can create identity providers in the given realm
    ///
    /// Requires ManageRealm permission on the target realm.
    async fn can_create_identity_provider(
        &self,
        identity: &Identity,
        realm_id: RealmId,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        // Get the user's realm to check access
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: realm_id,
            name: user_realm.name.clone(),
            display_name: None,
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission =
            Permissions::has_one_of_permissions(&permissions, &[Permissions::ManageRealm]);

        Ok(has_permission)
    }

    /// Checks if the identity can view the given identity provider
    ///
    /// Requires ViewRealm or ManageRealm permission on the provider's realm.
    async fn can_view_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == provider.realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: provider.realm_id,
            name: user_realm.name.clone(),
            display_name: None,
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ViewRealm],
        );

        Ok(has_permission)
    }

    /// Checks if the identity can update the given identity provider
    ///
    /// Requires ManageRealm permission on the provider's realm.
    async fn can_update_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        // Check realm access: same realm OR user is from master realm
        let is_same_realm = user_realm.id == provider.realm_id;
        let is_master_realm = user_realm.name == "master";

        if !is_same_realm && !is_master_realm {
            return Ok(false);
        }

        // Build a temporary realm for permission lookup
        let target_realm = Realm {
            id: provider.realm_id,
            name: user_realm.name.clone(),
            display_name: None,
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission =
            Permissions::has_one_of_permissions(&permissions, &[Permissions::ManageRealm]);

        Ok(has_permission)
    }

    /// Checks if the identity can delete the given identity provider
    ///
    /// Requires ManageRealm permission on the provider's realm.
    async fn can_delete_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> Result<bool, CoreError> {
        // Delete has the same requirements as update
        self.can_update_identity_provider(identity, provider).await
    }
}
