//! `FerriskeyPolicy`, the `Policy` trait and `ensure_policy` now live in the shared
//! `ferriskey-domain` crate (with all of `FerriskeyPolicy`'s domain-policy impls). Re-exported
//! here so existing `crate::domain::common::policies::{FerriskeyPolicy, Policy, ensure_policy}`
//! call sites keep compiling.
pub use ferriskey_domain::common::policies::{FerriskeyPolicy, Policy, ensure_policy};
