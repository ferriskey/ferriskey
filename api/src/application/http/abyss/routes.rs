use crate::application::{
    auth::auth,
    http::{
        abyss::federation::handlers::{
            create_provider::{__path_create_provider, create_provider},
            delete_provider::{__path_delete_provider, delete_provider},
            get_provider::{__path_get_provider, get_provider},
            list_providers::{__path_list_providers, list_providers},
            sync_users::{__path_sync_users, sync_users},
            test_connection::{__path_test_connection, test_connection},
            update_provider::{__path_update_provider, update_provider},
        },
        server::app_state::AppState,
    },
};
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_provider,
        get_provider,
        update_provider,
        delete_provider,
        list_providers,
        test_connection,
        sync_users,
    ),

    tags(
        (name = "Federation", description = "Federation provider management")
    )
)]
pub struct AbyssApiDoc;

pub fn abyss_routes(state: AppState) -> Router<AppState> {
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
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
