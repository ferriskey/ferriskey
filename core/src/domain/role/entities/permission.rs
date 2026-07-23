//! `Permissions` now lives in the zero-dependency kernel crate `ferriskey-domain`.
//! Re-exported here so existing `crate::domain::role::entities::permission::Permissions`
//! call sites keep compiling.
pub use ferriskey_domain::role::permission::Permissions;
