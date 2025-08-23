use crate::application::http::health::handlers::health_live::health_live;
use crate::application::http::health::handlers::health_ready::health_ready;
use crate::application::http::server::app_state::AppState;
use axum::{Router, routing::get};

pub mod handlers;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
}
