use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    add_member::{__path_add_member, add_member},
    create_organization::{__path_create_organization, create_organization},
    delete_attribute::{__path_delete_attribute, delete_attribute},
    delete_organization::{__path_delete_organization, delete_organization},
    get_organization::{__path_get_organization, get_organization},
    groups::{
        __path_add_group_member, __path_assign_group_role, __path_create_group,
        __path_delete_group, __path_delete_group_attribute, __path_get_group,
        __path_list_group_attributes, __path_list_group_members, __path_list_group_roles,
        __path_list_groups, __path_remove_group_member, __path_revoke_group_role,
        __path_update_group, __path_upsert_group_attribute, add_group_member, assign_group_role,
        create_group, delete_group, delete_group_attribute, get_group, list_group_attributes,
        list_group_members, list_group_roles, list_groups, remove_group_member, revoke_group_role,
        update_group, upsert_group_attribute,
    },
    list_attributes::{__path_list_attributes, list_attributes},
    list_members::{__path_list_members, list_members},
    list_organizations::{__path_list_organizations, list_organizations},
    member_roles::{
        __path_assign_member_role, __path_list_member_roles, __path_revoke_member_role,
        assign_member_role, list_member_roles, revoke_member_role,
    },
    remove_member::{__path_remove_member, remove_member},
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
    list_members,
    add_member,
    remove_member,
    list_member_roles,
    assign_member_role,
    revoke_member_role,
    list_groups,
    create_group,
    get_group,
    update_group,
    delete_group,
    list_group_members,
    add_group_member,
    remove_group_member,
    list_group_roles,
    assign_group_role,
    revoke_group_role,
    list_group_attributes,
    upsert_group_attribute,
    delete_group_attribute,
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
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/members",
                state.args.server.root_path
            ),
            get(list_members).post(add_member),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/members/{{user_id}}",
                state.args.server.root_path
            ),
            axum::routing::delete(remove_member),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/members/{{user_id}}/roles",
                state.args.server.root_path
            ),
            get(list_member_roles).post(assign_member_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/members/{{user_id}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            axum::routing::delete(revoke_member_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups",
                state.args.server.root_path
            ),
            get(list_groups).post(create_group),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}",
                state.args.server.root_path
            ),
            get(get_group)
                .put(update_group)
                .delete(delete_group),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/members",
                state.args.server.root_path
            ),
            get(list_group_members).post(add_group_member),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/members/{{user_id}}",
                state.args.server.root_path
            ),
            axum::routing::delete(remove_group_member),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/roles",
                state.args.server.root_path
            ),
            get(list_group_roles).post(assign_group_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/roles/{{role_id}}",
                state.args.server.root_path
            ),
            axum::routing::delete(revoke_group_role),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/attributes",
                state.args.server.root_path
            ),
            get(list_group_attributes),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/organizations/{{organization_id}}/groups/{{group_id}}/attributes/{{key}}",
                state.args.server.root_path
            ),
            axum::routing::put(upsert_group_attribute).delete(delete_group_attribute),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
