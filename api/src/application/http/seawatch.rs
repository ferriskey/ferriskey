//! The `seawatch` HTTP feature now lives in the `ferriskey-api-seawatch` crate. Re-exported here so
//! existing `crate::application::http::seawatch::{router::seawatch_router, router::SeawatchApiDoc}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_seawatch::*;
