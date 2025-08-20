use crate::{
    application::common::policies::PolicyEnforcer,
    domain::{
        authentication::value_objects::Identity, client::ports::ClientService,
        realm::entities::Realm, role::entities::permission::Permissions, user::ports::UserService,
        webhook::entities::WebhookError,
    },
};

pub struct WebhookPolicy;

impl WebhookPolicy {
    /// Check if the user has permission to delete a client.
    ///
    /// # Arguments
    /// * `identity` - The identity of the user.
    /// * `target_realm` - The realm of the client to be deleted.
    /// * `user_service` - The service for user operations.
    /// * `client_service` - The service for client operations.
    ///
    /// # Returns
    /// * `Ok(true)` if the user has permission to delete the client.
    /// * `Ok(false)` if the user does not have permission to delete the client.
    /// * `Err(WebErr)` if an error occurs during the operation.
    pub async fn delete<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, WebhookError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }

    pub async fn create<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, WebhookError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }

    pub async fn view<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, WebhookError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ViewClients],
        );

        Ok(has_permission)
    }

    pub async fn update<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, WebhookError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| WebhookError::InternalServerError)?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageWebhooks],
        );

        Ok(has_permission)
    }
}
