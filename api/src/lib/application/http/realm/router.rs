use crate::application::http::realm::handlers::create_realm::{__path_create_realm, create_realm};
use crate::application::http::realm::handlers::delete_realm::{__path_delete_realm, delete_realm};
use crate::application::http::realm::handlers::fetch_realm::{__path_fetch_realm, fetch_realm};
use crate::application::http::realm::handlers::get_realm::{__path_get_realm, get_realm};
use crate::application::http::realm::handlers::update_realm::{__path_update_realm, update_realm};
use crate::application::http::realm::handlers::update_realm_setting::{
    __path_update_realm_setting, update_realm_setting,
};
use crate::domain::realm::ports::RealmService;
use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    fetch_realm,
    get_realm,
    create_realm,
    update_realm,
    delete_realm,
    update_realm_setting
))]
pub struct RealmApiDoc;

pub fn realm_routes<R: RealmService>() -> Router {
    Router::new()
        .typed_get(fetch_realm::<R>)
        .typed_get(get_realm::<R>)
        .typed_post(create_realm::<R>)
        .typed_put(update_realm::<R>)
        .typed_delete(delete_realm::<R>)
        .typed_put(update_realm_setting::<R>)
}
