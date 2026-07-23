//! Role domain port traits live in the shared `ferriskey-domain` crate.
//! `RoleRepository` uses `#[cfg_attr(any(test, feature = "mock"), mockall::automock)]`; `core`
//! enables ferriskey-domain's `mock` feature, so `MockRoleRepository` comes through this glob
//! re-export — same automock convention as user/realm/client repository ports.
pub use ferriskey_domain::role::ports::*;
