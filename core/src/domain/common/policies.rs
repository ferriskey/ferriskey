use std::collections::HashSet;

use crate::domain::{
    authentication::value_objects::Identity, client::entities::Client,
    common::entities::app_errors::CoreError, realm::entities::Realm,
    role::entities::permission::Permissions, user::entities::User,
};

pub trait Policy: Clone + Send + Sync + 'static {
    fn get_user_from_identity(
        &self,
        identity: &Identity,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;
    fn get_user_permissions(
        &self,
        user: &User,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool;
    fn is_cross_realm_access(&self, user_realm: &Realm, target_realm: &Realm) -> bool;
}
