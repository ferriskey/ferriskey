use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    compass::ports::CompassPolicy,
    realm::entities::Realm,
    role::entities::permission::Permissions,
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> CompassPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_view_flows(&self, identity: &Identity, realm: &Realm) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(identity).await?;

        let permissions = self.get_permission_for_target_realm(&user, realm).await?;

        let has_permissions = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(has_permissions)
    }
}
