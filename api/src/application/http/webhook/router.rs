use super::handlers::create_webhook::{__path_create_webhook, create_webhook};
use super::handlers::delete_webhook::{__path_delete_webhook, delete_webhook};
use super::handlers::fetch_webhook::{__path_fetch_webhooks, fetch_webhooks};
use super::handlers::get_webhook::{__path_get_webhook, get_webhook};
use super::handlers::update_webhook::{__path_update_webhook, update_webhook};
use crate::application::{auth::auth, http::server::app_state::AppState};

use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
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
        .typed_get(fetch_webhooks)
        .typed_get(get_webhook)
        .typed_post(create_webhook)
        .typed_put(update_webhook)
        .typed_delete(delete_webhook)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
