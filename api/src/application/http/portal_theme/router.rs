use super::handlers::activate_theme::{__path_activate_theme, activate_theme};
use super::handlers::create_theme::{__path_create_theme, create_theme};
use super::handlers::delete_theme::{__path_delete_theme, delete_theme};
use super::handlers::get_active_theme::{__path_get_active_theme, get_active_theme};
use super::handlers::get_page_requirements::{__path_get_page_requirements, get_page_requirements};
use super::handlers::get_theme::{__path_get_theme, get_theme};
use super::handlers::get_theme_by_id::{__path_get_theme_by_id, get_theme_by_id};
use super::handlers::list_themes::{__path_list_themes, list_themes};
use super::handlers::update_theme::{__path_update_theme, update_theme};
use super::handlers::update_theme_metadata::{__path_update_theme_metadata, update_theme_metadata};
use super::handlers::update_theme_page::{__path_update_theme_page, update_theme_page};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{
    Router, middleware,
    routing::{get, post, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    get_theme,
    update_theme,
    list_themes,
    create_theme,
    get_theme_by_id,
    update_theme_metadata,
    delete_theme,
    update_theme_page,
    activate_theme,
    get_page_requirements,
))]
pub struct PortalThemeApiDoc;

#[derive(OpenApi)]
#[openapi(paths(get_active_theme))]
pub struct PortalThemePublicApiDoc;

pub fn portal_theme_routes(state: AppState) -> Router<AppState> {
    let admin_routes = Router::new()
        // Legacy single-theme endpoint (kept until cleanup PR).
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/theme",
                state.args.server.root_path
            ),
            get(get_theme).put(update_theme),
        )
        // Collection endpoints.
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/themes",
                state.args.server.root_path
            ),
            get(list_themes).post(create_theme),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/themes/{{theme_id}}",
                state.args.server.root_path
            ),
            get(get_theme_by_id)
                .put(update_theme_metadata)
                .delete(delete_theme),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/themes/{{theme_id}}/activate",
                state.args.server.root_path
            ),
            post(activate_theme),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/themes/{{theme_id}}/pages/{{page_type}}",
                state.args.server.root_path
            ),
            put(update_theme_page),
        )
        .route(
            &format!("{}/portal/page-requirements", state.args.server.root_path),
            get(get_page_requirements),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let public_routes = Router::new().route(
        &format!(
            "{}/realms/{{realm_name}}/portal/active",
            state.args.server.root_path
        ),
        get(get_active_theme),
    );

    admin_routes.merge(public_routes)
}
