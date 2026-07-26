//! `HashResult` moved to the shared `ferriskey-domain` crate so the `credential` domain (also in
//! `ferriskey-domain`) can reference it without a `ferriskey-domain -> ferriskey-security` cycle.
//! Re-exported here so `ferriskey_security::crypto::entities::HashResult` call sites keep resolving.
pub use ferriskey_domain::crypto::HashResult;
