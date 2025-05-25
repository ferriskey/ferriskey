use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::http::server::app_state::AppState;

use super::handlers::{
    assign_role::{__path_assign_role, assign_role},
    bulk_delete_user::{__path_bulk_delete_user, bulk_delete_user},
    create_user::{__path_create_user, create_user},
    get_user::{__path_get_user, get_user},
    get_users::{__path_get_users, get_users},
    reset_password::{__path_reset_password, reset_password},
    update_user::{__path_update_user, update_user},
};

#[derive(OpenApi)]
#[openapi(paths(
    get_users,
    get_user,
    assign_role,
    create_user,
    update_user,
    bulk_delete_user,
    reset_password,
))]
pub struct UserApiDoc;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .typed_get(get_users)
        .typed_get(get_user)
        .typed_post(create_user)
        .typed_put(update_user)
        .typed_put(reset_password)
        .typed_delete(bulk_delete_user)
        .typed_post(assign_role)
}
