//! `CompassServiceImpl` now lives in the `ferriskey-compass` lib crate (pure business logic,
//! generic over the repository ports — no SeaORM). Re-exported so existing
//! `crate::domain::compass::services::*` call sites keep compiling.
pub use ferriskey_compass::services::*;
