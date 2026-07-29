//! The `aegis` HTTP feature now lives in the `ferriskey-api-aegis` crate. Re-exported here so
//! existing `crate::application::http::aegis::router::{AegisApiDoc, aegis_routes}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_aegis::*;
