use axum::{
    Router, middleware,
    routing::{delete, get, patch, post, put},
};
use utoipa::OpenApi;

use super::handlers::{
    assign_default_scope::{__path_assign_default_scope, assign_default_scope},
    assign_optional_scope::{__path_assign_optional_scope, assign_optional_scope},
    create_client_scope::{__path_create_client_scope, create_client_scope},
    create_protocol_mapper::{__path_create_protocol_mapper, create_protocol_mapper},
    delete_client_scope::{__path_delete_client_scope, delete_client_scope},
    delete_protocol_mapper::{__path_delete_protocol_mapper, delete_protocol_mapper},
    get_client_scope::{__path_get_client_scope, get_client_scope},
    get_client_scopes::{__path_get_client_scopes, get_client_scopes},
    unassign_default_scope::{__path_unassign_default_scope, unassign_default_scope},
    unassign_optional_scope::{__path_unassign_optional_scope, unassign_optional_scope},
    update_client_scope::{__path_update_client_scope, update_client_scope},
    update_protocol_mapper::{__path_update_protocol_mapper, update_protocol_mapper},
};
use crate::application::{auth::auth, http::server::app_state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_client_scope,
        get_client_scopes,
        get_client_scope,
        update_client_scope,
        delete_client_scope,
        create_protocol_mapper,
        update_protocol_mapper,
        delete_protocol_mapper,
        assign_default_scope,
        unassign_default_scope,
        assign_optional_scope,
        unassign_optional_scope,
    ),
    tags(
        (name = "client-scope", description = "Client scope management")
    )
)]
pub struct AegisApiDoc;

pub fn aegis_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes",
                state.args.server.root_path
            ),
            get(get_client_scopes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes",
                state.args.server.root_path
            ),
            post(create_client_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            get(get_client_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            patch(update_client_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            delete(delete_client_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}/protocol-mappers",
                state.args.server.root_path
            ),
            post(create_protocol_mapper),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}/protocol-mappers/{{mapper_id}}",
                state.args.server.root_path
            ),
            patch(update_protocol_mapper),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/client-scopes/{{scope_id}}/protocol-mappers/{{mapper_id}}",
                state.args.server.root_path
            ),
            delete(delete_protocol_mapper),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/default-client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            put(assign_default_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/default-client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            delete(unassign_default_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/optional-client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            put(assign_optional_scope),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/clients/{{client_id}}/optional-client-scopes/{{scope_id}}",
                state.args.server.root_path
            ),
            delete(unassign_optional_scope),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
