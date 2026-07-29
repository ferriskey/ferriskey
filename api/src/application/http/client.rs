//! The `client` HTTP feature now lives in the `ferriskey-api-client` crate. Re-exported here so
//! existing `crate::application::http::client::router::{ClientApiDoc, client_routes}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_client::*;
