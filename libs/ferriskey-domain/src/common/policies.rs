use std::collections::HashSet;

use crate::auth::Identity;
use crate::client::entities::Client;
use crate::common::app_errors::CoreError;
use crate::realm::Realm;
use crate::role::permission::Permissions;
use crate::user::entities::User;

pub trait Policy: Send + Sync {
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

pub fn ensure_policy(
    result_has_permission: Result<bool, CoreError>,
    error_message: &str,
) -> Result<(), CoreError> {
    match result_has_permission {
        Ok(true) => Ok(()),
        Ok(false) => Err(CoreError::Forbidden(error_message.to_string())),
        Err(_) => Err(CoreError::Forbidden(error_message.to_string())),
    }
}
