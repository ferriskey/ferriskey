//! The `role` HTTP feature now lives in the `ferriskey-api-role` crate. Re-exported here so
//! existing `crate::application::http::role::router::{RoleApiDoc, role_routes}`
//! composition sites (http_server.rs, openapi.rs) keep resolving.
pub use ferriskey_api_role::*;
