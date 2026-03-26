use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    create_organization::{__path_create_organization, create_organization},
    delete_attribute::{__path_delete_attribute, delete_attribute},
    delete_organization::{__path_delete_organization, delete_organization},
    get_organization::{__path_get_organization, get_organization},
    list_attributes::{__path_list_attributes, list_attributes},
    list_organizations::{__path_list_organizations, list_organizations},
    update_organization::{__path_update_organization, update_organization},
    upsert_attribute::{__path_upsert_attribute, upsert_attribute},
};

#[derive(OpenApi)]
#[openapi(paths(
    list_organizations,
    create_organization,
    get_organization,
    update_organization,
    delete_organization,
    list_attributes,
    upsert_attribute,
    delete_attribute,
))]
pub struct OrganizationApiDoc;

pub fn organization_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations",
                state.args.server.root_path
            ),
            get(list_organizations).post(create_organization),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}",
                state.args.server.root_path
            ),
            get(get_organization)
                .put(update_organization)
                .delete(delete_organization),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/attributes",
                state.args.server.root_path
            ),
            get(list_attributes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/attributes/{{key}}",
                state.args.server.root_path
            ),
            axum::routing::put(upsert_attribute).delete(delete_attribute),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
