use axum::Router;
use axum_extra::routing::RouterExt;
use create_realm::create_realm;
use delete_realm::delete_realm;
use get_realm::get_realm;

use crate::domain::realm::ports::RealmService;

pub mod create_realm;
pub mod delete_realm;
pub mod get_realm;

pub fn realm_routes<R: RealmService>() -> Router {
    Router::new()
        .typed_post(create_realm::<R>)
        .typed_delete(delete_realm::<R>)
        .typed_get(get_realm::<R>)
}
