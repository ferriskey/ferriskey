//! User domain port traits now live in the shared `ferriskey-domain` crate.
//! Re-exported here so existing `crate::domain::user::ports::*` call sites keep compiling.
//! The repository traits keep `#[cfg_attr(any(test, feature = "mock"), mockall::automock)]` in
//! `ferriskey-domain`; `core` enables that crate's `mock` feature so `MockUserRepository` &co.
//! stay available to core's tests — same wiring as `ferriskey-organization`/`-abyss`/`-security`.
pub use ferriskey_domain::user::ports::*;
