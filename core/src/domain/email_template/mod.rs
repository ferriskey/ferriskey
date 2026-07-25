//! The email-template domain now lives in the `ferriskey-mail` lib crate (folded in as the
//! `email_template` module) — entities, ports, the `EmailTemplatePolicy` impl and the generic
//! `EmailTemplateServiceImpl` (pure business logic, no SeaORM). The SeaORM-backed repository stays
//! in `core` under `infrastructure/`. Re-exported here so existing `crate::domain::email_template::*`
//! call sites keep compiling.
pub use ferriskey_mail::email_template::*;
