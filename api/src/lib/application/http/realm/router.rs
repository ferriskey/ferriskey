use crate::application::http::realm::handlers::create_realm::__path_create_realm;
use crate::application::http::realm::handlers::create_realm::create_realm;
use crate::application::http::realm::handlers::delete_realm::delete_realm;
use crate::application::http::realm::handlers::get_realm::get_realm;
use crate::domain::realm::ports::RealmService;
use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
  paths(create_realm),
  tags(
        (name = "realm", description = "Realm management")
  )
)]
pub struct RealmApiDoc;

pub fn realm_routes<R: RealmService>() -> Router {
    Router::new()
        .typed_post(create_realm::<R>)
        .typed_delete(delete_realm::<R>)
        .typed_get(get_realm::<R>)
}
