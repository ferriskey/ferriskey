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
            burn_recovery_code::{__path_burn_recovery_code, burn_recovery_code},
            challenge_otp::{__path_challenge_otp, challenge_otp},
            challenge_webauthn::{__path_challenge_webauthn, challenge_webauthn},
            generate_recovery_codes::{__path_generate_recovery_codes, generate_recovery_codes},
            setup_otp::{__path_setup_otp, setup_otp},
            update_password::{__path_update_password, update_password},
            verify_otp::{__path_verify_otp, verify_otp},
            verify_webauthn::{__path_verify_webauthn, verify_webauthn},
        },
    },
};

#[derive(OpenApi)]
#[openapi(paths(
    setup_otp,
    verify_otp,
    challenge_otp,
    challenge_webauthn,
    verify_webauthn,
    update_password,
    burn_recovery_code,
    generate_recovery_codes
))]
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
                "{}/realms/{{realm_name}}/login-actions/challenge-webauthn",
                state.args.server.root_path
            ),
            post(challenge_webauthn),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/verify-webauthn",
                state.args.server.root_path
            ),
            post(verify_webauthn),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/update-password",
                state.args.server.root_path
            ),
            post(update_password),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/generate-recovery-codes",
                state.args.server.root_path
            ),
            post(generate_recovery_codes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/burn-recovery-code",
                state.args.server.root_path
            ),
            post(burn_recovery_code),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
