use axum::Router;
use axum_extra::routing::RouterExt;
use crate::application::http::health::handlers::health_ready::health_ready;
use crate::application::http::health::handlers::health_live::health_live;
use crate::application::http::server::app_state::AppState;

pub mod handlers;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .typed_get(health_ready)
        .typed_get(health_live)
}