use super::handlers::add_client_whitelist_entry::{
    __path_add_client_whitelist_entry, add_client_whitelist_entry,
};
use super::handlers::add_realm_whitelist_entry::{
    __path_add_realm_whitelist_entry, add_realm_whitelist_entry,
};
use super::handlers::get_client_whitelist::{__path_get_client_whitelist, get_client_whitelist};
use super::handlers::get_realm_whitelist::{__path_get_realm_whitelist, get_realm_whitelist};
use super::handlers::remove_client_whitelist_entry::{
    __path_remove_client_whitelist_entry, remove_client_whitelist_entry,
};
use super::handlers::remove_realm_whitelist_entry::{
    __path_remove_realm_whitelist_entry, remove_realm_whitelist_entry,
};
use super::handlers::toggle_maintenance::{__path_toggle_maintenance, toggle_maintenance};
use crate::application::{auth::auth, http::server::app_state::AppState};

use axum::{
    Router, middleware,
    routing::{delete, get, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    toggle_maintenance,
    get_client_whitelist,
    add_client_whitelist_entry,
    remove_client_whitelist_entry,
    get_realm_whitelist,
    add_realm_whitelist_entry,
    remove_realm_whitelist_entry,
))]
pub struct MaintenanceApiDoc;

pub fn maintenance_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Client maintenance toggle
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/maintenance",
                state.args.server.root_path
            ),
            put(toggle_maintenance),
        )
        // Client whitelist CRUD
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/maintenance/whitelist",
                state.args.server.root_path
            ),
            get(get_client_whitelist).post(add_client_whitelist_entry),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/maintenance/whitelist/{{entry_id}}",
                state.args.server.root_path
            ),
            delete(remove_client_whitelist_entry),
        )
        // Realm whitelist CRUD
        .route(
            &format!(
                "{}/realms/{{realm_name}}/settings/maintenance/whitelist",
                state.args.server.root_path
            ),
            get(get_realm_whitelist).post(add_realm_whitelist_entry),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/settings/maintenance/whitelist/{{entry_id}}",
                state.args.server.root_path
            ),
            delete(remove_realm_whitelist_entry),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
