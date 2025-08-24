use super::handlers::create_webhook::{__path_create_webhook, create_webhook};
use super::handlers::delete_webhook::{__path_delete_webhook, delete_webhook};
use super::handlers::fetch_webhook::{__path_fetch_webhooks, fetch_webhooks};
use super::handlers::get_webhook::{__path_get_webhook, get_webhook};
use super::handlers::update_webhook::{__path_update_webhook, update_webhook};
use crate::application::{auth::auth, http::server::app_state::AppState};

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    fetch_webhooks,
    get_webhook,
    create_webhook,
    update_webhook,
    delete_webhook
))]
pub struct WebhookApiDoc;

pub fn webhook_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/webhooks",
                state.args.server.root_path
            ),
            get(fetch_webhooks),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/webhooks/{{webhook_id}}",
                state.args.server.root_path
            ),
            get(get_webhook),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/webhooks",
                state.args.server.root_path
            ),
            post(create_webhook),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/webhooks/{{webhook_id}}",
                state.args.server.root_path
            ),
            put(update_webhook),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/webhooks/{{webhook_id}}",
                state.args.server.root_path
            ),
            delete(delete_webhook),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
