use ferriskey_authz::FerriskeyPolicy;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::Policy;
use ferriskey_domain::realm::Realm;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::OrganizationPolicy;

/// Organizations are administered through permissions of their own.
///
/// They used to borrow the realm's user permissions, which conflated two
/// different authorities: administering a realm's users is not the same as
/// administering its organizations. `ManageRealm` still grants everything, so
/// realm administrators are unaffected.
impl<U, C, UR> OrganizationPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_organization(
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
            &[Permissions::ManageRealm, Permissions::ManageOrganizations],
        ))
    }

    async fn can_view_organization(
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
            &[
                Permissions::ManageRealm,
                Permissions::ManageOrganizations,
                Permissions::ViewOrganizations,
            ],
        ))
    }

    async fn can_update_organization(
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
            &[Permissions::ManageRealm, Permissions::ManageOrganizations],
        ))
    }

    async fn can_delete_organization(
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
            &[Permissions::ManageRealm, Permissions::ManageOrganizations],
        ))
    }

    async fn can_manage_members(
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
            &[Permissions::ManageRealm, Permissions::ManageOrganizations],
        ))
    }
}
