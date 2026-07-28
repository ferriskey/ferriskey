//! The `health` HTTP feature now lives in the `ferriskey-api-health` crate. Re-exported here so
//! existing `crate::application::http::health::health_routes` composition sites (http_server.rs)
//! keep resolving.
pub use ferriskey_api_health::*;
