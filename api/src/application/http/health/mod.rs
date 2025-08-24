use crate::application::http::health::handlers::health_live::health_live;
use crate::application::http::health::handlers::health_ready::health_ready;
use crate::application::http::server::app_state::AppState;
use axum::{Router, routing::get};

pub mod handlers;

pub fn health_routes(root_path: &str) -> Router<AppState> {
    Router::new()
        .route(&format!("{root_path}/health/ready"), get(health_ready))
        .route(&format!("{root_path}/health/live"), get(health_live))
}
