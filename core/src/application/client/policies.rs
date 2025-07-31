use crate::{
    application::common::policies::PolicyEnforcer,
    domain::{
        authentication::value_objects::Identity,
        client::{entities::ClientError, ports::ClientService},
        realm::entities::Realm,
        role::entities::permission::Permissions,
        user::ports::UserService,
    },
};

pub struct ClientPolicy;

impl ClientPolicy {
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
    /// * `Err(ClientError)` if an error occurs during the operation.
    pub async fn delete<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, ClientError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }
}
