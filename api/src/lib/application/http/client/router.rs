use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::{
    create_client::{__path_create_client, create_client},
    create_redirect_uri::{__path_create_redirect_uri, create_redirect_uri},
    get_redirect_uris::{__path_get_redirect_uris, get_redirect_uris},
};
use crate::application::http::server::app_state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_client,
        create_redirect_uri,
        get_redirect_uris
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
        .typed_get(get_redirect_uris)
}
