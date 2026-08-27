use axum::{
    Router, middleware,
    routing::{delete, get, patch, post, put},
};
use utoipa::OpenApi;

use super::handlers::{
    create_client::{__path_create_client, create_client},
    create_post_logout_redirect_uri::{
        __path_create_post_logout_redirect_uri, create_post_logout_redirect_uri,
    },
    create_redirect_uri::{__path_create_redirect_uri, create_redirect_uri},
    create_role::{__path_create_role, create_role},
    create_saml_attribute_mapper::{
        __path_create_saml_attribute_mapper, create_saml_attribute_mapper,
    },
    create_web_origin::{__path_create_web_origin, create_web_origin},
    delete_client::{__path_delete_client, delete_client},
    delete_post_logout_redirect_uri::{
        __path_delete_post_logout_redirect_uri, delete_post_logout_redirect_uri,
    },
    delete_redirect_uri::{__path_delete_redirect_uri, delete_redirect_uri},
    delete_saml_attribute_mapper::{
        __path_delete_saml_attribute_mapper, delete_saml_attribute_mapper,
    },
    delete_web_origin::{__path_delete_web_origin, delete_web_origin},
    evaluate_scopes::{__path_evaluate_scopes, evaluate_scopes},
    get_client::{__path_get_client, get_client},
    get_client_roles::{__path_get_client_roles, get_client_roles},
    get_client_saml_config::{__path_get_client_saml_config, get_client_saml_config},
    get_client_secret::{__path_get_client_secret, get_client_secret},
    get_clients::{__path_get_clients, get_clients},
    get_post_logout_redirect_uris::{
        __path_get_post_logout_redirect_uris, get_post_logout_redirect_uris,
    },
    get_redirect_uris::{__path_get_redirect_uris, get_redirect_uris},
    get_saml_attribute_mappers::{__path_get_saml_attribute_mappers, get_saml_attribute_mappers},
    get_web_origins::{__path_get_web_origins, get_web_origins},
    set_client_saml_config::{__path_set_client_saml_config, set_client_saml_config},
    update_client::{__path_update_client, update_client},
    update_post_logout_redirect_uri::{
        __path_update_post_logout_redirect_uri, update_post_logout_redirect_uri,
    },
    update_redirect_uri::{__path_update_redirect_uri, update_redirect_uri},
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::auth;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_client,
        get_client_secret,
        get_clients,
        create_client,
        delete_client,
        create_redirect_uri,
        create_post_logout_redirect_uri,
        create_role,
        get_redirect_uris,
        get_post_logout_redirect_uris,
        update_client,
        update_redirect_uri,
        update_post_logout_redirect_uri,
        delete_redirect_uri,
        delete_post_logout_redirect_uri,
        get_client_roles,
        evaluate_scopes,
        create_web_origin,
        get_web_origins,
        delete_web_origin,
        get_client_saml_config,
        set_client_saml_config,
        create_saml_attribute_mapper,
        get_saml_attribute_mappers,
        delete_saml_attribute_mapper
    ),

    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients",
                state.args.server.root_path
            ),
            get(get_clients),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}",
                state.args.server.root_path
            ),
            get(get_client),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/client-secret",
                state.args.server.root_path
            ),
            get(get_client_secret),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients",
                state.args.server.root_path
            ),
            post(create_client),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}",
                state.args.server.root_path
            ),
            patch(update_client),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/redirects",
                state.args.server.root_path
            ),
            post(create_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/post-logout-redirects",
                state.args.server.root_path
            ),
            post(create_post_logout_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/roles",
                state.args.server.root_path
            ),
            post(create_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/redirects",
                state.args.server.root_path
            ),
            get(get_redirect_uris),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/post-logout-redirects",
                state.args.server.root_path
            ),
            get(get_post_logout_redirect_uris),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/redirects/{{uri_id}}",
                state.args.server.root_path
            ),
            put(update_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/post-logout-redirects/{{uri_id}}",
                state.args.server.root_path
            ),
            put(update_post_logout_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}",
                state.args.server.root_path
            ),
            delete(delete_client),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/redirects/{{uri_id}}",
                state.args.server.root_path
            ),
            delete(delete_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/post-logout-redirects/{{uri_id}}",
                state.args.server.root_path
            ),
            delete(delete_post_logout_redirect_uri),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/roles",
                state.args.server.root_path
            ),
            get(get_client_roles),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/evaluate-scopes",
                state.args.server.root_path
            ),
            post(evaluate_scopes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/web-origins",
                state.args.server.root_path
            ),
            post(create_web_origin),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/web-origins",
                state.args.server.root_path
            ),
            get(get_web_origins),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/web-origins/{{web_origin_id}}",
                state.args.server.root_path
            ),
            delete(delete_web_origin),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/saml-config",
                state.args.server.root_path
            ),
            get(get_client_saml_config),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/saml-config",
                state.args.server.root_path
            ),
            put(set_client_saml_config),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/saml-attribute-mappers",
                state.args.server.root_path
            ),
            post(create_saml_attribute_mapper),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/saml-attribute-mappers",
                state.args.server.root_path
            ),
            get(get_saml_attribute_mappers),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/saml-attribute-mappers/{{mapper_id}}",
                state.args.server.root_path
            ),
            delete(delete_saml_attribute_mapper),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
