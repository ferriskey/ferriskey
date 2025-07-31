use crate::{
    application::common::policies::PolicyEnforcer,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
        realm::entities::Realm,
        role::entities::{RoleError, permission::Permissions},
        user::ports::UserService,
    },
};

pub struct RolePolicy {}

impl RolePolicy {
    /// Check if the user has the necessary permissions to create a role in the target realm.
    ///
    /// # Arguments
    /// * `identity` - The identity of the user.
    /// * `target_realm` - The target realm.
    /// * `user_service` - The user service.
    /// * `client_service` - The client service.
    ///
    /// # Returns
    /// * `Ok(true)` if the user has the necessary permissions.
    /// * `Ok(false)` if the user does not have the necessary permissions.
    /// * `Err(RoleError)` if an error occurs.
    pub async fn create<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RoleError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageUsers, Permissions::ManageRealm],
        ))
    }

    pub async fn view<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RoleError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[
                Permissions::ViewRoles,
                Permissions::ManageRealm,
                Permissions::ManageRoles,
            ],
        ))
    }

    pub async fn update<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RoleError>
    where
        U: UserService,
        C: ClientService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RoleError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RoleError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRoles, Permissions::ManageRealm],
        ))
    }
}
