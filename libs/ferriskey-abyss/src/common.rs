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
/// Single source of truth now lives in the kernel crate `ferriskey-domain`.
/// Re-exported so existing `crate::common::ensure_policy` call sites keep compiling.
pub use ferriskey_domain::common::policies::ensure_policy;
