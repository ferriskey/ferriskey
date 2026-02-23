use std::future::Future;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::{Realm, RealmId};

/// Minimal trait for resolving realms by name or ID.
///
/// This trait abstracts the realm lookup needed by abyss services,
/// allowing them to be decoupled from the full `RealmRepository` in core.
pub trait RealmResolver: Send + Sync {
    fn get_realm_by_name(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;

    fn get_realm_by_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;
}

/// Helper to convert a policy check result into a `Result<(), CoreError>`.
///
/// Returns `Ok(())` if the policy returned `Ok(true)`,
/// otherwise returns `Err(CoreError::Forbidden(...))`.
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
