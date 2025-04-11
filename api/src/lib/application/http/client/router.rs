use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::create_client::{__path_create_client, create_client};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(create_client),
    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes() -> Router<AppState> {
    Router::new().typed_post(create_client)
}
