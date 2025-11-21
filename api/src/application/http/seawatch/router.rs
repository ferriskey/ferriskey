use axum::{Router, middleware};
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
pub struct SeawatchApiDoc;

pub fn seawatch_router(state: AppState) -> Router<AppState> {
    Router::new().layer(middleware::from_fn_with_state(state.clone(), auth))
}
