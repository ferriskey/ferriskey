//! The portal-theme domain now lives in the `ferriskey-portal-theme` lib crate — entities,
//! validation, ports, the `PortalThemePolicy` impl and the generic `PortalThemeServiceImpl` (pure
//! business logic, no SeaORM). The SeaORM-backed repository stays in `core` under
//! `infrastructure/`. Re-exported here so existing `crate::domain::portal_theme::*` call sites keep
//! compiling.
pub use ferriskey_portal_theme::*;
