use axum::Router;
use axum_extra::routing::RouterExt;

use crate::{
    application::http::authentication::handlers::token::exchange_token,
    domain::{
        authentication::ports::AuthenticationService, client::ports::ClientService,
        realm::ports::RealmService,
    },
};

use super::handlers::auth::auth;

pub fn authentication_routes<A, R, C>() -> Router
where
    A: AuthenticationService,
    R: RealmService,
    C: ClientService,
{
    Router::new()
        .typed_post(exchange_token::<A>)
        .typed_get(auth::<R, C>)
}
