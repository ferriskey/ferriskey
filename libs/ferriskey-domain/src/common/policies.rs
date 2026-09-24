//! Authorization contracts.
//!
//! The kernel owns the contracts; the engine that satisfies them lives in
//! `ferriskey-authz`. Keeping [`Policy`] and [`ensure_policy`] here is what
//! lets kernel services (credential, session) express an authorization check
//! without the kernel depending on the authorization crate.

use std::collections::HashSet;

use crate::auth::Identity;
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
    fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<HashSet<Permissions>, CoreError>> + Send;
    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool;
}

/// A well-formed question whose answer is "no" is a 403. A question the
/// engine couldn't answer — a database read failing, a realm that doesn't
/// resolve — is not the same thing and must not be reported as one: it
/// propagates as its own `CoreError`, whatever HTTP status that maps to.
/// Collapsing both into `Forbidden` would hide infrastructure failures
/// behind an authorization error, indistinguishable from a legitimate deny.
pub fn ensure_policy(
    result_has_permission: Result<bool, CoreError>,
    error_message: &str,
) -> Result<(), CoreError> {
    if result_has_permission? {
        Ok(())
    } else {
        Err(CoreError::Forbidden(error_message.to_string()))
    }
}
