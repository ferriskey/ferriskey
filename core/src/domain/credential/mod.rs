//! The credential domain now lives in the shared `ferriskey-domain` crate — entities (backed by
//! `webauthn_rs` types), ports and the generic `CredentialServiceImpl` (pure business logic, no
//! SeaORM). The SeaORM-backed repository stays in `core` under `infrastructure/`. Re-exported here
//! so existing `crate::domain::credential::*` call sites keep compiling.
pub use ferriskey_domain::credential::*;
