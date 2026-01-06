use crate::application::http::server::app_state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn abyss_routes() -> Router<AppState> {
    Router::new().nest("/federation", federation_routes())
}

fn federation_routes() -> Router<AppState> {
    use super::federation::handlers;

    Router::new()
        .route("/providers", post(handlers::create_provider))
        .route("/providers", get(handlers::list_providers))
        .route("/providers/:id", get(handlers::get_provider))
        .route("/providers/:id", put(handlers::update_provider))
        .route("/providers/:id", delete(handlers::delete_provider))
}
