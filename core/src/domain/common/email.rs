//! The `EmailPort` trait now lives in the shared `ferriskey-domain` crate (it only references
//! `CoreError` and `realm::SmtpConfig`, both now in `ferriskey-domain`). Re-exported so existing
//! `crate::domain::common::email::{EmailPort, MockEmailPort}` call sites keep resolving.
pub use ferriskey_domain::common::email::*;
