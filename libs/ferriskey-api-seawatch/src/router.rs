use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

use crate::handlers::get_security_events::{__path_get_security_events, get_security_events};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::auth;

#[derive(OpenApi)]
#[openapi(paths(get_security_events))]
pub struct SeawatchApiDoc;

pub fn seawatch_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/seawatch/v1/security-events",
                state.args.server.root_path
            ),
            get(get_security_events),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
