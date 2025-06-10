use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::{
    auth::{__path_auth, auth},
    authentificate::{__path_authenticate, authenticate},
    get_certs::{__path_get_certs, get_certs},
    token::{__path_exchange_token, exchange_token},
};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(paths(exchange_token, authenticate, get_certs, auth))]
pub struct AuthenticationApiDoc;

pub fn authentication_routes() -> Router<AppState> {
    Router::new()
        .typed_post(exchange_token)
        .typed_get(auth)
        .typed_post(authenticate)
        .typed_get(get_certs)
}
