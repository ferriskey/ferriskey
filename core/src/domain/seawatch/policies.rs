use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    realm::entities::Realm,
    role::entities::permission::Permissions,
    seawatch::SecurityEventPolicy,
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> SecurityEventPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_export_events(&self, identity: Identity, realm: Realm) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self.get_permission_for_target_realm(&user, &realm).await?;

        let has_permissions = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(has_permissions)
    }

    async fn can_view_events(&self, identity: Identity, realm: Realm) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self.get_permission_for_target_realm(&user, &realm).await?;

        let has_permissions = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(has_permissions)
    }
}
