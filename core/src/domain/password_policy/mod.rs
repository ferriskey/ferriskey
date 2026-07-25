//! The password-policy domain now lives in the `ferriskey-password-policy` lib crate — entity,
//! errors, the CNIL-compliant validator, ports, the `PasswordPolicyPolicy` impl and the generic
//! `PasswordPolicyService` (pure business logic, no SeaORM). The SeaORM-backed repository stays in
//! `core` under `infrastructure/`. Re-exported here so existing `crate::domain::password_policy::*`
//! call sites keep compiling.
pub use ferriskey_password_policy::*;
