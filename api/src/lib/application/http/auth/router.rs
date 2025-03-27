use axum::Router;
use axum_extra::routing::RouterExt;

use crate::domain::{client::ports::ClientService, realm::ports::RealmService};

use super::handlers::auth::auth;

pub fn auth_router<R, C>() -> Router
where
    R: RealmService,
    C: ClientService,
{
    Router::new().typed_get(auth::<R, C>)
}
