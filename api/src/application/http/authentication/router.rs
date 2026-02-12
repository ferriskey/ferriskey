use axum::{
    Router, middleware,
    routing::{get, post},
};

use utoipa::OpenApi;

use super::handlers::{
    auth::{__path_auth_handler, auth_handler},
    authentificate::{__path_authenticate, authenticate},
    get_certs::{__path_get_certs, get_certs},
    introspect::{__path_introspect_token, introspect_token},
    logout::{__path_logout, logout},
    openid_configuration::{__path_get_openid_configuration, get_openid_configuration},
    registration::{__path_registration_handler, registration_handler},
    token::{__path_exchange_token, exchange_token},
    userinfo::{__path_get_userinfo, get_userinfo},
};
use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
#[openapi(paths(
    exchange_token,
    introspect_token,
    authenticate,
    get_certs,
    auth_handler,
    logout,
    get_openid_configuration,
    registration_handler,
    get_userinfo,
))]
pub struct AuthenticationApiDoc;

pub fn authentication_routes(state: AppState, root_path: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/userinfo"),
            get(get_userinfo),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/userinfo"),
            post(get_userinfo),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/token"),
            post(exchange_token),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/token/introspect"),
            post(introspect_token),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/logout"),
            get(logout),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/logout"),
            post(logout),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/auth"),
            get(auth_handler),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/registrations"),
            post(registration_handler),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/login-actions/authenticate"),
            post(authenticate),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/certs"),
            get(get_certs),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/.well-known/openid-configuration"),
            get(get_openid_configuration),
        )
}
