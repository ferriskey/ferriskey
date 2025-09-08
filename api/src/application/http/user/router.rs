use crate::application::auth::auth;
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

use crate::application::http::server::app_state::AppState;

use super::handlers::{
    assign_role::{__path_assign_role, assign_role},
    bulk_delete_user::{__path_bulk_delete_user, bulk_delete_user},
    create_user::{__path_create_user, create_user},
    delete_credential::{__path_delete_user_credential, delete_user_credential},
    delete_user::{__path_delete_user, delete_user},
    get_credentials::{__path_get_user_credentials, get_user_credentials},
    get_user::{__path_get_user, get_user},
    get_user_roles::{__path_get_user_roles, get_user_roles},
    get_users::{__path_get_users, get_users},
    reset_password::{__path_reset_password, reset_password},
    unassign_role::{__path_unassign_role, unassign_role},
    update_user::{__path_update_user, update_user},
};

#[derive(OpenApi)]
#[openapi(paths(
    get_users,
    get_user,
    get_user_roles,
    assign_role,
    create_user,
    update_user,
    bulk_delete_user,
    delete_user,
    reset_password,
    get_user_credentials,
    delete_user_credential,
    unassign_role,
))]
pub struct UserApiDoc;

pub fn user_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users",
                state.args.server.root_path
            ),
            get(get_users),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}",
                state.args.server.root_path
            ),
            get(get_user),
        )
        .route(
            "/realms/{realm_name}/users/{user_id}/roles",
            get(get_user_roles),
        )
        .route(
            "/realms/{realm_name}/users/{user_id}/credentials",
            get(get_user_credentials),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users",
                state.args.server.root_path
            ),
            post(create_user),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}",
                state.args.server.root_path
            ),
            put(update_user),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}/reset-password",
                state.args.server.root_path
            ),
            put(reset_password),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/bulk",
                state.args.server.root_path
            ),
            delete(bulk_delete_user),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}",
                state.args.server.root_path
            ),
            delete(delete_user),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}/credentials/{{credential_id}}",
                state.args.server.root_path
            ),
            delete(delete_user_credential),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            post(assign_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/users/{{user_id}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            delete(unassign_role),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
