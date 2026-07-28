use crate::handlers::health_live::health_live;
use crate::handlers::health_ready::health_ready;
use axum::{Router, routing::get};
use ferriskey_api_core::app_state::AppState;

pub mod handlers;

pub fn health_routes(root_path: &str) -> Router<AppState> {
    Router::new()
        .route(&format!("{root_path}/health/ready"), get(health_ready))
        .route(&format!("{root_path}/health/live"), get(health_live))
}
