use axum::{Router, routing::get};
use utoipa::OpenApi;

use ferriskey_api_core::app_state::AppState;

use crate::handlers::descriptor::__path_saml_descriptor;
use crate::handlers::sso::{__path_saml_sso_post, __path_saml_sso_redirect};
use crate::handlers::sso_continue::__path_saml_continue;
use crate::handlers::{saml_continue, saml_descriptor, saml_sso_post, saml_sso_redirect};

#[derive(OpenApi)]
#[openapi(paths(saml_descriptor, saml_sso_redirect, saml_sso_post, saml_continue))]
pub struct SamlApiDoc;

pub fn saml_routes(state: AppState, root_path: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/saml"),
            get(saml_sso_redirect).post(saml_sso_post),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/saml/continue"),
            get(saml_continue),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/saml/descriptor"),
            get(saml_descriptor),
        )
        .with_state(state)
}
