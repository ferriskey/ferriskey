//! The portal-layouts domain now lives in the `ferriskey-portal-layouts` lib crate — entities, ports,
//! the `PortalLayoutsPolicy` impl and the generic `PortalLayoutsServiceImpl` (pure business logic, no
//! SeaORM). The SeaORM-backed repository stays in `core` under `infrastructure/`. Re-exported here so
//! existing `crate::domain::portal_layouts::*` call sites keep compiling.
pub use ferriskey_portal_layouts::*;
