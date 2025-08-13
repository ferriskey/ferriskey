use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::{
    auth::auth,
    http::{
        server::app_state::AppState,
        trident::handlers::{
            challenge_otp::{__path_challenge_otp, challenge_otp},
            setup_otp::{__path_setup_otp, setup_otp},
            verify_otp::{__path_verify_otp, verify_otp},
        },
    },
};

#[derive(OpenApi)]
#[openapi(paths(setup_otp, verify_otp, challenge_otp))]
pub struct TridentApiDoc;

pub fn trident_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(setup_otp)
        .typed_post(verify_otp)
        .typed_post(challenge_otp)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
