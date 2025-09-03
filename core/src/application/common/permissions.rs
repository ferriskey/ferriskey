use std::collections::HashSet;

use crate::{
    domain::{
        authentication::value_objects::Identity,
        client::entities::Client,
        common::{entities::app_errors::CoreError, policies::Policy},
        realm::entities::Realm,
        role::entities::permission::Permissions,
        user::entities::User,
    },
    infrastructure::{client::repositories::ClientRepoAny, user::UserRepoAny},
};

#[derive(Clone)]
pub struct FerriskeyPolicy {
    user_repository: UserRepoAny,
    client_repository: ClientRepoAny,
}

impl Policy for FerriskeyPolicy {
    async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, CoreError> {
        todo!()
    }

    async fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> Result<HashSet<Permissions>, CoreError> {
        todo!()
    }

    async fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> Result<HashSet<Permissions>, CoreError> {
        todo!()
    }

    async fn get_user_permissions(&self, user: &User) -> Result<HashSet<Permissions>, CoreError> {
        todo!()
    }

    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        todo!()
    }

    fn is_cross_realm_access(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        todo!()
    }
}
