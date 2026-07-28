//! The `compass` HTTP feature now lives in the `ferriskey-api-compass` crate. Re-exported here so
//! existing `crate::application::http::compass::router::{CompassApiDoc, compass_routes}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_compass::*;
