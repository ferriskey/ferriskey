use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::authentificate::__path_authenticate;
use super::handlers::{auth::auth, authentificate::authenticate};
use crate::{
    application::http::authentication::handlers::token::{__path_exchange_token, exchange_token},
    domain::{
        authentication::ports::auth_session::AuthSessionService,
        authentication::ports::authentication::AuthenticationService,
        client::ports::client_service::ClientService, realm::ports::realm_service::RealmService,
    },
};

#[derive(OpenApi)]
#[openapi(paths(exchange_token, authenticate))]
pub struct AuthenticationApiDoc;

pub fn authentication_routes<A, R, C, AS>() -> Router
where
    A: AuthenticationService,
    R: RealmService,
    C: ClientService,
    AS: AuthSessionService,
{
    Router::new()
        .typed_post(exchange_token::<A>)
        .typed_get(auth::<R, C, AS>)
        .typed_post(authenticate::<A, AS>)
}
