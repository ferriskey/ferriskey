use axum::{Router, routing::get};
use utoipa::OpenApi;

use crate::handlers::get_consent::{__path_get_consent, get_consent};
use crate::handlers::post_consent::{__path_post_consent, post_consent};
use ferriskey_api_core::app_state::AppState;

#[derive(OpenApi)]
#[openapi(paths(get_consent, post_consent))]
pub struct ConsentApiDoc;

pub fn consent_router(state: AppState) -> Router<AppState> {
    Router::new().route(
        &format!(
            "{}/realms/{{realm_name}}/auth/consent",
            state.args.server.root_path
        ),
        get(get_consent).post(post_consent),
    )
}
