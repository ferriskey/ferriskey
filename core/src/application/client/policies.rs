use crate::{
    application::common::permissions::FerriskeyPolicy,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientPolicy,
        common::{entities::app_errors::CoreError, policies::Policy},
        realm::entities::Realm,
        role::entities::permission::Permissions,
    },
};

impl ClientPolicy for FerriskeyPolicy {
    async fn can_create_client(
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
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_delete_client(
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
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_update_client(
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
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    async fn can_view_client(
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
            &[Permissions::ManageRealm, Permissions::ViewClients],
        );

        Ok(has_permission)
    }
}
