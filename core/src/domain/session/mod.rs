//! The session domain now lives in the shared `ferriskey-domain` crate — entities, ports and the
//! generic `UserSessionServiceImpl` (pure business logic, no SeaORM). The SeaORM-backed repository
//! stays in `core` under `infrastructure/`. Re-exported here so existing
//! `crate::domain::session::*` call sites keep compiling.
pub use ferriskey_domain::session::*;
