//! The `Policy` trait and `ensure_policy` live in the kernel crate
//! `ferriskey-domain`; `FerriskeyPolicy`, the engine implementing them, lives in
//! `ferriskey-authz`. Re-exported here so existing
//! `crate::domain::common::policies::{FerriskeyPolicy, Policy, ensure_policy}`
//! call sites keep compiling.
pub use ferriskey_authz::FerriskeyPolicy;
pub use ferriskey_domain::common::policies::{Policy, ensure_policy};
