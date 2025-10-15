use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    realm::entities::Realm,
    role::{entities::permission::Permissions, ports::RolePolicy},
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> RolePolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_create_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_delete_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_update_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_view_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        tracing::info!("permissions: {:?}", permissions);

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[
                Permissions::ManageRealm,
                Permissions::ManageUsers,
                Permissions::ViewRoles,
            ],
        );

        Ok(has_permission)
    }
}
