use crate::application::{
    auth::auth,
    http::{
        abyss::federation::handlers::{
            create_provider::{__path_create_provider, create_provider},
            delete_provider::{__path_delete_provider, delete_provider},
            get_provider::{__path_get_provider, get_provider},
            list_providers::{__path_list_providers, list_providers},
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
        list_providers
    ),

    tags(
        (name = "federation", description = "Federation provider management")
    )
)]
pub struct AbyssApiDoc;

pub fn abyss_routes(state: AppState) -> Router<AppState> {
    let path = format!(
        "{}/realms/{{realm_name}}/federation",
        state.args.server.root_path
    );
    Router::new()
        .nest(&path, federation_routes())
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}

fn federation_routes() -> Router<AppState> {
    Router::new()
        .route("/providers", post(create_provider))
        .route("/providers", get(list_providers))
        .route("/providers/{{id}}", get(get_provider))
        .route("/providers/{{id}}", put(update_provider))
        .route("/providers/{{id}}", delete(delete_provider))
}
