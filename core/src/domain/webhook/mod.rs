//! The webhook domain now lives in the `ferriskey-webhook` lib crate — entities, ports, the
//! `WebhookPolicy` impl and the generic `WebhookServiceImpl` (pure business logic, no SeaORM). The
//! SeaORM-backed repository stays in `core` under `infrastructure/webhook/`. Re-exported here so
//! existing `crate::domain::webhook::*` call sites keep compiling.
pub use ferriskey_webhook::*;
