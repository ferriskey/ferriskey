use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::{
    create_client::{__path_create_client, create_client},
    create_redirect_uri::{__path_create_redirect_uri, create_redirect_uri},
};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_client,
        create_redirect_uri
    ),

    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes() -> Router<AppState> {
    Router::new()
        .typed_post(create_client)
        .typed_post(create_redirect_uri)
}
