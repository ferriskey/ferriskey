use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::application::http::{
    abyss::federation::handlers::{
        create_provider, delete_provider, get_provider, list_providers, sync_users::sync_users,
        test_connection, update_provider,
    },
    server::app_state::AppState,
};

pub mod dto;
pub mod handlers;

pub fn federation_routes(state: AppState) -> Router<AppState> {
    let root_path = format!(
        "{}/realms/{{realm_name}}/federation/providers",
        state.args.server.root_path
    );

    Router::new()
        .route(&root_path, post(create_provider))
        .route(&root_path, get(list_providers))
        .route(&format!("{}/{{id}}", root_path), get(get_provider))
        .route(&format!("{}/{{id}}", root_path), put(update_provider))
        .route(&format!("{}/{{id}}", root_path), delete(delete_provider))
        .route(
            &format!("{}/{{id}}/test-connection", root_path),
            post(test_connection),
        )
        .route(
            &format!("{}/{{id}}/sync-users", root_path),
            post(sync_users),
        )
}
