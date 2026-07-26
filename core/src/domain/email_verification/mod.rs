//! The email-verification domain now lives in the `ferriskey-mail` lib crate (folded in as the
//! `email_verification` module) — entities, ports and the generic `EmailVerificationServiceImpl`
//! (pure business logic, no SeaORM). The SeaORM-backed token repository stays in `core` under
//! `infrastructure/`. Re-exported here so existing `crate::domain::email_verification::*` call
//! sites keep compiling.
pub use ferriskey_mail::email_verification::*;
