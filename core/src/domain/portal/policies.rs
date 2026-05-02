use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    portal::ports::PortalPolicy,
    realm::entities::Realm,
    role::entities::permission::Permissions,
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> PortalPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_view_portal(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ManageRealm,
                Permissions::ManagePortal,
                Permissions::ViewPortal,
            ],
        );

        Ok(has_permission)
    }

    async fn can_manage_portal(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm, Permissions::ManagePortal],
        );

        Ok(has_permission)
    }
}
