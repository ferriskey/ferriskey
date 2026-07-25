use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::Realm;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::ports::SecurityEventPolicy;

impl<U, C, UR> SecurityEventPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_export_events(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self.get_permission_for_target_realm(&user, realm).await?;

        let has_permissions =
            Permissions::has_one_of_permissions(&permissions, &[Permissions::ManageRealm]);

        Ok(has_permissions)
    }

    async fn can_view_events(&self, identity: &Identity, realm: &Realm) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self.get_permission_for_target_realm(&user, realm).await?;

        let has_permissions =
            Permissions::has_one_of_permissions(&permissions, &[Permissions::ManageRealm]);

        Ok(has_permissions)
    }
}
