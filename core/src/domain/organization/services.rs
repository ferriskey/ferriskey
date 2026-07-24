//! `OrganizationServiceImpl` now lives in the `ferriskey-organization` lib crate (pure business
//! logic, generic over the repository ports — no SeaORM). Re-exported so existing
//! `crate::domain::organization::services::*` call sites keep compiling.
pub use ferriskey_organization::services::*;
