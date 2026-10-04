//! Authorization rules a realm writes for its own applications.
//!
//! Distinct from the rest of this crate, which guards FerrisKey's own admin
//! API: here FerrisKey is the policy decision point, and the rules belong to
//! the tenant.

pub mod entities;
pub mod names;
pub mod ports;
pub mod services;
pub mod value_objects;

mod policies;

/// Reserved for the types FerrisKey resolves itself (`FerrisKey::User`, …).
pub const FERRISKEY_NAMESPACE: &str = "FerrisKey";
