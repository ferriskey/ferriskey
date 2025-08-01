use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    get_role::{__path_get_role, get_role},
    get_roles::{__path_get_roles, get_roles},
    update_role::{__path_update_role, update_role},
    update_role_permissions::{__path_update_role_permissions, update_role_permissions},
};

#[derive(OpenApi)]
#[openapi(paths(get_roles, get_role, update_role, update_role_permissions))]
pub struct RoleApiDoc;

pub fn role_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_roles)
        .typed_get(get_role)
        .typed_put(update_role)
        .typed_patch(update_role_permissions)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
