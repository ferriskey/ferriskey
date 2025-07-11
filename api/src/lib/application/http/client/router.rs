use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::{
    create_client::{__path_create_client, create_client},
    create_redirect_uri::{__path_create_redirect_uri, create_redirect_uri},
    create_role::{__path_create_role, create_role},
    delete_client::{__path_delete_client, delete_client},
    delete_redirect_uri::{__path_delete_redirect_uri, delete_redirect_uri},
    get_client::{__path_get_client, get_client},
    get_client_roles::{__path_get_client_roles, get_client_roles},
    get_clients::{__path_get_clients, get_clients},
    get_redirect_uris::{__path_get_redirect_uris, get_redirect_uris},
    update_client::{__path_update_client, update_client},
    update_redirect_uri::{__path_update_redirect_uri, update_redirect_uri},
};
use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_client,
        get_clients,
        create_client,
        delete_client,
        create_redirect_uri,
        create_role,
        get_redirect_uris,
        update_client,
        update_redirect_uri,
        delete_redirect_uri,
        get_client_roles
    ),

    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_clients)
        .typed_get(get_client)
        .typed_post(create_client)
        .typed_patch(update_client)
        .typed_post(create_redirect_uri)
        .typed_post(create_role)
        .typed_get(get_redirect_uris)
        .typed_put(update_redirect_uri)
        .typed_delete(delete_client)
        .typed_delete(delete_redirect_uri)
        .typed_get(get_client_roles)
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
