//! The `webhook` HTTP feature now lives in the `ferriskey-api-webhook` crate. Re-exported here so
//! existing `crate::application::http::webhook::router::{WebhookApiDoc, webhook_routes}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_webhook::*;
