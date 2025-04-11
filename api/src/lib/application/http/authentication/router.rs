use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::authentificate::__path_authenticate;
use super::handlers::{auth::auth, authentificate::authenticate};
use crate::application::http::authentication::handlers::token::{
    __path_exchange_token, exchange_token,
};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(paths(exchange_token, authenticate))]
pub struct AuthenticationApiDoc;

pub fn authentication_routes() -> Router<AppState> {
    Router::new()
        .typed_post(exchange_token)
        .typed_get(auth)
        .typed_post(authenticate)
}
