use super::handlers::get_branding::{__path_get_branding, get_branding};
use super::handlers::get_public_branding::{__path_get_public_branding, get_public_branding};
use super::handlers::update_branding::{__path_update_branding, update_branding};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_branding, update_branding, get_public_branding))]
pub struct RealmBrandingApiDoc;

pub fn realm_branding_routes(state: AppState) -> Router<AppState> {
    let admin_routes = Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/branding",
                state.args.server.root_path
            ),
            get(get_branding).put(update_branding),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let public_routes = Router::new().route(
        &format!(
            "{}/realms/{{realm_name}}/branding/public",
            state.args.server.root_path
        ),
        get(get_public_branding),
    );

    admin_routes.merge(public_routes)
}
