//! The seawatch (audit) domain now lives in the `ferriskey-seawatch` lib crate — entities, the
//! tamper-evident hash chain, PII pseudonymisation, ports, the `SecurityEventPolicy` impl and the
//! generic `SecurityEventServiceImpl` (pure business logic, no SeaORM). The SeaORM-backed
//! repository stays in `core` under `infrastructure/seawatch/`. Re-exported here so existing
//! `crate::domain::seawatch::*` call sites keep compiling.
pub use ferriskey_seawatch::*;
