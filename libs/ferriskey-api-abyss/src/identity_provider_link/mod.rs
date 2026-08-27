use axum::{
    Router,
    routing::{delete, get},
};

use crate::identity_provider_link::handlers::{
    delete_identity_provider_link::delete_identity_provider_link,
    list_identity_provider_links::list_identity_provider_links,
};
use ferriskey_api_core::app_state::AppState;

pub mod dto;
pub mod handlers;

pub fn identity_provider_link_routes(state: AppState) -> Router<AppState> {
    let root_path = format!(
        "{}/realms/{{realm_name}}/users/{{user_id}}/identity-provider-links",
        state.args.server.root_path
    );

    Router::new()
        .route(&root_path, get(list_identity_provider_links))
        .route(
            &format!("{}/{{link_id}}", root_path),
            delete(delete_identity_provider_link),
        )
}
