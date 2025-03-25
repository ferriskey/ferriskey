use axum::Router;
use axum_extra::routing::RouterExt;

use super::handlers::auth::auth;

pub fn auth_router() -> Router {
    Router::new().typed_get(auth)
}
