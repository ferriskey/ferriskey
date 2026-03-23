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
            forgot_password::{__path_forgot_password, forgot_password},
            generate_recovery_codes::{__path_generate_recovery_codes, generate_recovery_codes},
            magic_link::{
                __path_send_magic_link, __path_verify_magic_link, send_magic_link,
                verify_magic_link,
            },
            passkey_authenticate::{__path_passkey_authenticate, passkey_authenticate},
            passkey_request_options::{__path_passkey_request_options, passkey_request_options},
            reset_password::{
                __path_reset_password_with_token, __path_verify_reset_token,
                reset_password_with_token, verify_reset_token,
            },
            setup_otp::{__path_setup_otp, setup_otp},
            update_password::{__path_update_password, update_password},
            verify_otp::{__path_verify_otp, verify_otp},
            webauthn_public_key_authenticate::{
                __path_webauthn_public_key_authenticate, webauthn_public_key_authenticate,
            },
            webauthn_public_key_create::{
                __path_webauthn_public_key_create, webauthn_public_key_create,
            },
            webauthn_public_key_create_options::{
                __path_webauthn_public_key_create_options, webauthn_public_key_create_options,
            },
            webauthn_public_key_request_options::{
                __path_webauthn_public_key_request_options, webauthn_public_key_request_options,
            },
        },
    },
};

#[derive(OpenApi)]
#[openapi(paths(
    setup_otp,
    verify_otp,
    challenge_otp,
    update_password,
    burn_recovery_code,
    generate_recovery_codes,
    webauthn_public_key_create,
    webauthn_public_key_create_options,
    webauthn_public_key_authenticate,
    webauthn_public_key_request_options,
    passkey_request_options,
    passkey_authenticate,
    send_magic_link,
    verify_magic_link,
    forgot_password,
    reset_password_with_token,
    verify_reset_token,
))]
pub struct TridentApiDoc;

pub fn trident_routes(state: AppState) -> Router<AppState> {
    // Public routes
    let public_routes = Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/send-magic-link",
                state.args.server.root_path
            ),
            post(send_magic_link),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/verify-magic-link",
                state.args.server.root_path
            ),
            get(verify_magic_link),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/forgot-password",
                state.args.server.root_path
            ),
            post(forgot_password),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/reset-password",
                state.args.server.root_path
            ),
            post(reset_password_with_token),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/verify-reset-token",
                state.args.server.root_path
            ),
            post(verify_reset_token),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/passkey-request-options",
                state.args.server.root_path
            ),
            post(passkey_request_options),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/passkey-authenticate",
                state.args.server.root_path
            ),
            post(passkey_authenticate),
        );

    // Authenticated routes
    let protected_routes = Router::new()
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
                "{}/realms/{{realm_name}}/login-actions/webauthn-public-key-create",
                state.args.server.root_path
            ),
            post(webauthn_public_key_create),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/webauthn-public-key-create-options",
                state.args.server.root_path
            ),
            post(webauthn_public_key_create_options),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/webauthn-public-key-request-options",
                state.args.server.root_path
            ),
            post(webauthn_public_key_request_options),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/webauthn-public-key-authenticate",
                state.args.server.root_path
            ),
            post(webauthn_public_key_authenticate),
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
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    // Merge both router
    public_routes.merge(protected_routes)
}
