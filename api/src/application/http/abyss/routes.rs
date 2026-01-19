use crate::application::{
    auth::auth,
    http::{
        abyss::{federation::federation_routes, identity_provider::identity_provider_routes},
        server::app_state::AppState,
    },
};
use axum::{Router, middleware};

pub fn abyss_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(federation_routes(state.clone()))
        .merge(identity_provider_routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
