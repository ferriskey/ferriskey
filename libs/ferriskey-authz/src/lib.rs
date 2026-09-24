//! Authorization domain for FerrisKey.
//!
//! This crate owns the decision side of authorization: the engine-agnostic
//! request/decision vocabulary, the ports an engine plugs into, and the
//! concrete RBAC engine ([`FerriskeyPolicy`]) FerrisKey ships today.
//!
//! The contracts stay in the kernel crate `ferriskey-domain`: the per-domain
//! `XxxPolicy` traits live in their own crate's `ports.rs`, and the `Policy`
//! trait plus `ensure_policy` stay in `ferriskey_domain::common::policies`.
//! Only the implementation lives here.

pub mod actions;
pub mod entities;
pub mod error;
pub mod matrix;
pub mod ports;

mod engine;

pub use engine::FerriskeyPolicy;
pub use error::AuthzError;
