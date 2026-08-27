use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use utoipa::OpenApi;

use crate::handlers::{
    burn_recovery_code::{__path_burn_recovery_code, burn_recovery_code},
    challenge_otp::{__path_challenge_otp, challenge_otp},
    forgot_password::{__path_forgot_password, forgot_password},
    generate_recovery_codes::{__path_generate_recovery_codes, generate_recovery_codes},
    magic_link::{
        __path_send_magic_link, __path_verify_magic_link, send_magic_link, verify_magic_link,
    },
    me_credentials::{
        __path_me_credentials, __path_me_delete_credential, me_credentials, me_delete_credential,
    },
    me_passkey_register::{__path_me_passkey_register, me_passkey_register},
    me_passkey_register_options::{
        __path_me_passkey_register_options, me_passkey_register_options,
    },
    me_reauthenticate::{__path_me_reauthenticate, me_reauthenticate},
    me_totp_setup::{__path_me_totp_setup, me_totp_setup},
    me_totp_verify::{__path_me_totp_verify, me_totp_verify},
    passkey_authenticate::{__path_passkey_authenticate, passkey_authenticate},
    passkey_request_options::{__path_passkey_request_options, passkey_request_options},
    reset_password::{
        __path_reset_password_with_token, __path_verify_reset_token, reset_password_with_token,
        verify_reset_token,
    },
    reset_password_with_recovery_code::{
        __path_reset_password_with_recovery_code, reset_password_with_recovery_code,
    },
    setup_otp::{__path_setup_otp, setup_otp},
    update_password::{__path_update_password, update_password},
    verify_otp::{__path_verify_otp, verify_otp},
    webauthn_public_key_authenticate::{
        __path_webauthn_public_key_authenticate, webauthn_public_key_authenticate,
    },
    webauthn_public_key_create::{__path_webauthn_public_key_create, webauthn_public_key_create},
    webauthn_public_key_create_options::{
        __path_webauthn_public_key_create_options, webauthn_public_key_create_options,
    },
    webauthn_public_key_request_options::{
        __path_webauthn_public_key_request_options, webauthn_public_key_request_options,
    },
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::{auth, auth_login_actions};

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
    reset_password_with_recovery_code,
    me_totp_setup,
    me_totp_verify,
    me_reauthenticate,
    me_credentials,
    me_delete_credential,
    me_passkey_register_options,
    me_passkey_register,
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
                "{}/realms/{{realm_name}}/login-actions/reset-password-with-recovery-code",
                state.args.server.root_path
            ),
            post(reset_password_with_recovery_code),
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

    // Login action routes (protected by temporary token)
    let login_action_routes = Router::new()
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
        .route(
            &format!(
                "{}/realms/{{realm_name}}/login-actions/webauthn-public-key-create-options",
                state.args.server.root_path
            ),
            post(webauthn_public_key_create_options),
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
                "{}/realms/{{realm_name}}/login-actions/burn-recovery-code",
                state.args.server.root_path
            ),
            post(burn_recovery_code),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_login_actions,
        ));

    // Bearer authenticated routes
    let bearer_protected_routes = Router::new()
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
                "{}/realms/{{realm_name}}/login-actions/generate-recovery-codes",
                state.args.server.root_path
            ),
            post(generate_recovery_codes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/totp/setup",
                state.args.server.root_path
            ),
            post(me_totp_setup),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/totp/verify",
                state.args.server.root_path
            ),
            post(me_totp_verify),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/reauthenticate",
                state.args.server.root_path
            ),
            post(me_reauthenticate),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/credentials",
                state.args.server.root_path
            ),
            get(me_credentials),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/credentials/{{credential_id}}",
                state.args.server.root_path
            ),
            delete(me_delete_credential),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/passkey/registration-options",
                state.args.server.root_path
            ),
            post(me_passkey_register_options),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/me/passkey/registration",
                state.args.server.root_path
            ),
            post(me_passkey_register),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    // Merge all routers
    public_routes
        .merge(login_action_routes)
        .merge(bearer_protected_routes)
}
