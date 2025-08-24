use axum::{
    Router, middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::application::{
    auth::auth,
    http::{
        server::app_state::AppState,
        trident::handlers::{
            challenge_otp::{__path_challenge_otp, challenge_otp},
            setup_otp::{__path_setup_otp, setup_otp},
            update_password::{__path_update_password, update_password},
            verify_otp::{__path_verify_otp, verify_otp},
        },
    },
};

#[derive(OpenApi)]
#[openapi(paths(setup_otp, verify_otp, challenge_otp, update_password))]
pub struct TridentApiDoc;

pub fn trident_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/setup-otp",
                state.args.server.root_path
            ),
            get(setup_otp),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/verify-otp",
                state.args.server.root_path
            ),
            post(verify_otp),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/challenge-otp",
                state.args.server.root_path
            ),
            post(challenge_otp),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/update-password",
                state.args.server.root_path
            ),
            post(update_password),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
