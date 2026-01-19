use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::application::http::{
    abyss::identity_provider::handlers::{
        create_identity_provider::create_identity_provider,
        delete_identity_provider::delete_identity_provider,
        get_identity_provider::get_identity_provider,
        list_identity_providers::list_identity_providers,
        update_identity_provider::update_identity_provider,
    },
    server::app_state::AppState,
};

pub mod dto;
pub mod handlers;

pub fn identity_provider_routes(state: AppState) -> Router<AppState> {
    let root_path = format!(
        "{}/realms/{{realm_name}}/identity-providers",
        state.args.server.root_path
    );

    Router::new()
        .route(&root_path, post(create_identity_provider))
        .route(&root_path, get(list_identity_providers))
        .route(&format!("{}/{{id}}", root_path), get(get_identity_provider))
        .route(
            &format!("{}/{{id}}", root_path),
            put(update_identity_provider),
        )
        .route(
            &format!("{}/{{id}}", root_path),
            delete(delete_identity_provider),
        )
}
