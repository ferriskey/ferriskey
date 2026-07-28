//! The `broker` HTTP feature now lives in the `ferriskey-api-broker` crate. Re-exported here so
//! existing `crate::application::http::broker::{BrokerApiDoc, router::broker_routes}` composition
//! sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_broker::*;
