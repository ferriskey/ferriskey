//! Compass — authentication flow tracing for the admin dashboard.
//!
//! # Best-effort, by design
//!
//! Compass answers "where do logins get stuck?", not "what happened to this
//! account?". Events are handed to a bounded channel with `try_send` and are
//! **dropped when the writer falls behind**, so a missing flow or step means
//! "the server was busy", never "it did not happen". Rows are also purged on a
//! retention window (see [`ports::CompassFlowRepository::purge_old_flows`]),
//! and tracing is opt-out per realm via `realm_settings.compass_enabled`.
//!
//! None of that is acceptable for an audit trail. Anything that has to survive
//! backpressure, retention and the toggle belongs in `ferriskey-seawatch`,
//! which writes synchronously and chains its records.

pub mod entities;
pub mod ports;
pub mod recorder;
pub mod value_objects;

pub mod policies;
pub mod services;
