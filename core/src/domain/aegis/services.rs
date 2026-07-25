//! The aegis service impls (`ClientScopeServiceImpl`, `ProtocolMapperServiceImpl`,
//! `ScopeMappingServiceImpl`) now live in the `ferriskey-aegis` lib crate (pure business logic,
//! generic over the repository ports — no SeaORM). Re-exported so existing
//! `crate::domain::aegis::services::*` call sites keep compiling.
pub use ferriskey_aegis::services::*;
