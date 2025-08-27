use axum::{
    Router, middleware,
    routing::{delete, get, patch, put},
};
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    delete_role::{__path_delete_role, delete_role},
    get_role::{__path_get_role, get_role},
    get_roles::{__path_get_roles, get_roles},
    update_role::{__path_update_role, update_role},
    update_role_permissions::{__path_update_role_permissions, update_role_permissions},
};

#[derive(OpenApi)]
#[openapi(paths(get_roles, get_role, update_role, update_role_permissions, delete_role))]
pub struct RoleApiDoc;

pub fn role_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/roles",
                state.args.server.root_path
            ),
            get(get_roles),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            get(get_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            delete(delete_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/roles/{{role_id}}/permissions",
                state.args.server.root_path
            ),
            put(update_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/roles/{{role_id}}/permissions",
                state.args.server.root_path
            ),
            patch(update_role_permissions),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
