use crate::application::auth::auth;
use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::http::{
    server::app_state::AppState,
    trident::handlers::setup_otp::{__path_setup_otp, setup_otp},
};

#[derive(OpenApi)]
#[openapi(paths(setup_otp))]
pub struct TridentApiDoc;

pub fn trident_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(setup_otp)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
