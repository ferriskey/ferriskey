use crate::application::common::policies::PolicyEnforcer;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::ports::OldClientService;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::role::entities::permission::Permissions;
use crate::domain::user::ports::UserService;

pub struct RealmPolicy;

impl RealmPolicy {
    pub async fn create<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RealmError>
    where
        C: OldClientService,
        U: UserService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }

    pub async fn delete<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RealmError>
    where
        C: OldClientService,
        U: UserService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }

    pub async fn view<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RealmError>
    where
        C: OldClientService,
        U: UserService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ViewRealm, Permissions::ManageRealm],
        ))
    }

    pub async fn update<C, U>(
        identity: Identity,
        target_realm: Realm,
        user_service: U,
        client_service: C,
    ) -> Result<bool, RealmError>
    where
        C: OldClientService,
        U: UserService,
    {
        let policy = PolicyEnforcer::new(user_service, client_service);
        let user = policy
            .get_user_from_identity(&identity)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        let permissions: Vec<Permissions> = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .into_iter()
            .collect();

        Ok(Permissions::has_one_of_permissions(
            &permissions,
            &[Permissions::ManageRealm],
        ))
    }
}
