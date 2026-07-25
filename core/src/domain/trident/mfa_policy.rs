//! The MFA enforcement policy helpers (`user_requires_mfa`, `required_action_for_mfa`) now live in
//! the `ferriskey-trident` lib crate (pure functions over `ferriskey-domain` types — no
//! infrastructure). Re-exported so existing `crate::domain::trident::mfa_policy::*` call sites keep
//! compiling.
pub use ferriskey_trident::mfa_policy::*;
