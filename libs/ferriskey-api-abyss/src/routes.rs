use crate::{federation::federation_routes, identity_provider::identity_provider_routes};
use axum::{Router, middleware};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::auth;

pub fn abyss_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(federation_routes(state.clone()))
        .merge(identity_provider_routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
