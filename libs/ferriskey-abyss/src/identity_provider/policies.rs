use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::Realm;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::identity_provider::IdentityProviderPolicy;

/// FK-006. Each method used to hand-roll its own realm guard and then fabricate a
/// `Realm` for the permission lookup:
///
/// ```ignore
/// let target_realm = Realm { id: realm_id, name: user_realm.name.clone(), .. };
/// self.get_permission_for_target_realm(&user, &target_realm)
/// ```
///
/// Copying the *caller's* name into the target made `is_cross_realm_access` — which
/// tests `user_realm.name == "master" && user_realm.name != target_realm.name` —
/// compare a string to its own clone, so it was always false. The lookup therefore
/// took the `else` branch and returned `get_user_permissions`: the unscoped union of
/// every role the caller holds, in any realm. A master user carrying `ManageRealm` on
/// realm A administered the identity providers of realm B.
///
/// Passing the real realm restores both gates and removes the manual guard entirely.
/// `FederationPolicy`, in the same subdomain, was already written this way.
impl<U, C, UR> IdentityProviderPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    /// Requires `ManageRealm` on the target realm.
    async fn can_create_identity_provider(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }

    /// Requires `ManageRealm` or `ViewRealm` on the target realm.
    async fn can_view_identity_provider(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ViewRealm],
        ))
    }

    /// Requires `ManageRealm` on the target realm.
    async fn can_update_identity_provider(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }

    /// Delete has the same requirements as update.
    async fn can_delete_identity_provider(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        self.can_update_identity_provider(identity, target_realm)
            .await
    }
}
