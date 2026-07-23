//! `GroupServiceImpl` now lives in the `ferriskey-organization` lib crate. Re-exported so existing
//! `crate::domain::organization::group_services::*` call sites keep compiling.
pub use ferriskey_organization::group_services::*;
