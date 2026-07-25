use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy};
use ferriskey_domain::realm::Realm;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::ports::WebhookPolicy;

impl<U, C, UR> WebhookPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_view_webhook(
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
                Permissions::ManageWebhooks,
                Permissions::ViewWebhooks,
            ],
        );

        Ok(has_permission)
    }

    async fn can_create_webhook(
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
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }

    async fn can_update_webhook(
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
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }

    async fn can_delete_webhook(
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
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }
}
