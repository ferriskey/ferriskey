use axum::{
    Router, middleware,
    routing::{delete, get, patch, post, put},
};
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
        .route("/realms/{realm_name}/clients", get(get_clients))
        .route("/realms/{realm_name}/clients/{client_id}", get(get_client))
        .route("/realms/{realm_name}/clients", post(create_client))
        .route(
            "/realms/{realm_name}/clients/{client_id}",
            patch(update_client),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/redirects",
            post(create_redirect_uri),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/roles",
            post(create_role),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/redirects",
            get(get_redirect_uris),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}",
            put(update_redirect_uri),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}",
            delete(delete_client),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}",
            delete(delete_redirect_uri),
        )
        .route(
            "/realms/{realm_name}/clients/{client_id}/roles",
            get(get_client_roles),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
