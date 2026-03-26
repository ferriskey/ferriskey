use super::handlers::activate_template::{__path_activate_template, activate_template};
use super::handlers::create_template::{__path_create_template, create_template};
use super::handlers::delete_template::{__path_delete_template, delete_template};
use super::handlers::fetch_templates::{__path_fetch_templates, fetch_templates};
use super::handlers::get_template::{__path_get_template, get_template};
use super::handlers::get_variables::{__path_get_variables, get_variables};
use super::handlers::update_template::{__path_update_template, update_template};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    fetch_templates,
    get_template,
    create_template,
    update_template,
    delete_template,
    activate_template,
))]
pub struct EmailTemplateApiDoc;

#[derive(OpenApi)]
#[openapi(paths(get_variables))]
pub struct EmailTemplateVariablesApiDoc;

pub fn email_template_routes(state: AppState) -> Router<AppState> {
    let template_routes = Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/email-templates",
                state.args.server.root_path
            ),
            get(fetch_templates).post(create_template),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/email-templates/{{template_id}}",
                state.args.server.root_path
            ),
            get(get_template)
                .put(update_template)
                .delete(delete_template),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/email-templates/{{template_id}}/activate",
                state.args.server.root_path
            ),
            axum::routing::patch(activate_template),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let variables_routes = Router::new().route(
        &format!(
            "{}/email-templates/variables/{{email_type}}",
            state.args.server.root_path
        ),
        get(get_variables),
    );

    template_routes.merge(variables_routes)
}
