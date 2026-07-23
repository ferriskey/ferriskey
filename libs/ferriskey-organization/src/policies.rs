use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::{Organization, OrganizationPolicy};

impl<U, C, UR> OrganizationPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_organization(
        &self,
        identity: &Identity,
        _realm_id: RealmId,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;
        let permissions = self.get_user_permissions(&user).await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        ))
    }

    async fn can_view_organization(
        &self,
        identity: &Identity,
        _organization: &Organization,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;
        let permissions = self.get_user_permissions(&user).await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ManageRealm,
                Permissions::ManageUsers,
                Permissions::ViewUsers,
            ],
        ))
    }

    async fn can_update_organization(
        &self,
        identity: &Identity,
        _organization: &Organization,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;
        let permissions = self.get_user_permissions(&user).await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        ))
    }

    async fn can_delete_organization(
        &self,
        identity: &Identity,
        _organization: &Organization,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;
        let permissions = self.get_user_permissions(&user).await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }

    async fn can_manage_members(
        &self,
        identity: &Identity,
        _organization: &Organization,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;
        let permissions = self.get_user_permissions(&user).await?;

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        ))
    }
}
