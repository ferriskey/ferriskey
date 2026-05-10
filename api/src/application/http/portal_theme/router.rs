use super::handlers::get_theme::{__path_get_theme, get_theme};
use super::handlers::update_theme::{__path_update_theme, update_theme};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_theme, update_theme))]
pub struct PortalThemeApiDoc;

pub fn portal_theme_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/portal/theme",
                state.args.server.root_path
            ),
            get(get_theme).put(update_theme),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
