use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::{
    create_client::{__path_create_client, create_client},
    create_redirect_uri::{__path_create_redirect_uri, create_redirect_uri},
    delete_redirect_uri::{__path_delete_redirect_uri, delete_redirect_uri},
    get_clients::get_clients,
    get_redirect_uris::{__path_get_redirect_uris, get_redirect_uris},
    update_redirect_uri::{__path_update_redirect_uri, update_redirect_uri},
};
use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_client,
        create_redirect_uri,
        get_redirect_uris,
        update_redirect_uri,
        delete_redirect_uri

    ),

    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_clients)
        .typed_post(create_client)
        .typed_post(create_redirect_uri)
        .typed_get(get_redirect_uris)
        .typed_put(update_redirect_uri)
        .typed_delete(delete_redirect_uri)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
