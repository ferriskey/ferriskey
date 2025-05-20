use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    create_role::{__path_create_role, create_role},
    get_roles::{__path_get_roles, get_roles},
};

#[derive(OpenApi)]
#[openapi(paths(create_role, get_roles))]
pub struct RoleApiDoc;

pub fn role_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_roles)
        .typed_post(create_role)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
