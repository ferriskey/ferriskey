use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::auth;
use utoipa::OpenApi;

use super::handlers::{
    change_own_password::{__path_change_own_password, change_own_password},
    confirm_own_otp_enrollment::{__path_confirm_own_otp_enrollment, confirm_own_otp_enrollment},
    confirm_own_passkey_registration::{
        __path_confirm_own_passkey_registration, confirm_own_passkey_registration,
    },
    delete_own_passkey::{__path_delete_own_passkey, delete_own_passkey},
    disable_own_otp::{__path_disable_own_otp, disable_own_otp},
    get_own_profile::{__path_get_own_profile, get_own_profile},
    list_own_credentials::{__path_list_own_credentials, list_own_credentials},
    request_elevation::{__path_request_elevation, request_elevation},
    start_own_otp_enrollment::{__path_start_own_otp_enrollment, start_own_otp_enrollment},
    start_own_passkey_registration::{
        __path_start_own_passkey_registration, start_own_passkey_registration,
    },
    update_me_locale::{__path_update_me_locale, update_me_locale},
    update_own_profile::{__path_update_own_profile, update_own_profile},
};

#[derive(OpenApi)]
#[openapi(paths(
    get_own_profile,
    update_own_profile,
    update_me_locale,
    request_elevation,
    change_own_password,
    list_own_credentials,
    start_own_otp_enrollment,
    confirm_own_otp_enrollment,
    disable_own_otp,
    start_own_passkey_registration,
    confirm_own_passkey_registration,
    delete_own_passkey,
))]
pub struct AccountApiDoc;

pub fn account_routes(state: AppState) -> Router<AppState> {
    let root = &state.args.server.root_path;

    Router::new()
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me"),
            get(get_own_profile).put(update_own_profile),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/@me/locale"),
            put(update_me_locale),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/reauthenticate"),
            post(request_elevation),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/password"),
            put(change_own_password),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/credentials"),
            get(list_own_credentials),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/mfa/otp"),
            post(start_own_otp_enrollment)
                .put(confirm_own_otp_enrollment)
                .delete(disable_own_otp),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/passkeys/options"),
            post(start_own_passkey_registration),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/passkeys"),
            post(confirm_own_passkey_registration),
        )
        .route(
            &format!("{root}/realms/{{realm_name}}/users/me/passkeys/{{credential_id}}"),
            delete(delete_own_passkey),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
