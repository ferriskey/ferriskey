use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::Realm;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::identity_provider::IdentityProviderPolicy;

impl<U, C, UR> IdentityProviderPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
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

    async fn can_delete_identity_provider(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        self.can_update_identity_provider(identity, target_realm)
            .await
    }
}
