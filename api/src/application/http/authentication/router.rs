use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;

use super::handlers::{
    auth::{__path_auth, auth},
    authentificate::{__path_authenticate, authenticate},
    get_certs::{__path_get_certs, get_certs},
    openid_configuration::{__path_get_openid_configuration, get_openid_configuration},
    token::{__path_exchange_token, exchange_token},
};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(paths(
    exchange_token,
    authenticate,
    get_certs,
    auth,
    get_openid_configuration
))]
pub struct AuthenticationApiDoc;

pub fn authentication_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/realms/{realm_name}/protocol/openid-connect/token",
            post(exchange_token),
        )
        .route(
            "/realms/{realm_name}/protocol/openid-connect/auth",
            get(auth),
        )
        .route(
            "/realms/{realm_name}/login-actions/authenticate",
            post(authenticate),
        )
        .route(
            "/realms/{realm_name}/protocol/openid-connect/certs",
            get(get_certs),
        )
        .route(
            "/realms/{realm_name}/.well-known/openid-configuration",
            get(get_openid_configuration),
        )
}
