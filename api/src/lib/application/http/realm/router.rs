use crate::application::auth::auth;
use crate::application::http::realm::handlers::create_realm::{__path_create_realm, create_realm};
use crate::application::http::realm::handlers::delete_realm::{__path_delete_realm, delete_realm};
use crate::application::http::realm::handlers::fetch_realm::{__path_fetch_realm, fetch_realm};
use crate::application::http::realm::handlers::get_realm::{__path_get_realm, get_realm};
use crate::application::http::realm::handlers::update_realm::{__path_update_realm, update_realm};
use crate::application::http::realm::handlers::update_realm_setting::{
    __path_update_realm_setting, update_realm_setting,
};
use crate::application::http::server::app_state::AppState;
use axum::{Router, middleware};
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

pub fn realm_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(fetch_realm)
        .typed_get(get_realm)
        .typed_post(create_realm)
        .typed_put(update_realm)
        .typed_delete(delete_realm)
        .typed_put(update_realm_setting)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
