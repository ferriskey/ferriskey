use axum::Router;
use axum_extra::routing::RouterExt;

use crate::{
    application::http::authentication::handlers::token::exchange_token,
    domain::authentication::ports::AuthenticationService,
};

pub fn authentication_routes<A: AuthenticationService>() -> Router {
    Router::new().typed_post(exchange_token::<A>)
}
