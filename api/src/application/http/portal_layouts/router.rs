use super::handlers::create_layout::{__path_create_layout, create_layout};
use super::handlers::delete_layout::{__path_delete_layout, delete_layout};
use super::handlers::get_layout::{__path_get_layout, get_layout};
use super::handlers::get_public_default_layout::{
    __path_get_public_default_layout, get_public_default_layout,
};
use super::handlers::get_public_layout::{__path_get_public_layout, get_public_layout};
use super::handlers::list_layouts::{__path_list_layouts, list_layouts};
use super::handlers::set_default_layout::{__path_set_default_layout, set_default_layout};
use super::handlers::update_layout::{__path_update_layout, update_layout};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    list_layouts,
    create_layout,
    get_layout,
    update_layout,
    delete_layout,
    set_default_layout,
))]
pub struct PortalLayoutsApiDoc;

#[derive(OpenApi)]
#[openapi(paths(get_public_default_layout, get_public_layout))]
pub struct PortalLayoutsPublicApiDoc;

pub fn portal_layouts_routes(state: AppState) -> Router<AppState> {
    let admin_routes = Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal-layouts",
                state.args.server.root_path
            ),
            get(list_layouts).post(create_layout),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal-layouts/{{layout_id}}",
                state.args.server.root_path
            ),
            get(get_layout).put(update_layout).delete(delete_layout),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal-layouts/{{layout_id}}/default",
                state.args.server.root_path
            ),
            axum::routing::put(set_default_layout),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let public_routes = Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal-layouts/public/default",
                state.args.server.root_path
            ),
            get(get_public_default_layout),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal-layouts/public/{{layout_id}}",
                state.args.server.root_path
            ),
            get(get_public_layout),
        );

    admin_routes.merge(public_routes)
}
