use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
// Import ONLY the member-role service trait so `state.service.{assign,revoke,list}_role`
// resolves unambiguously to `OrganizationMemberRoleService` (GroupService / OrganizationService
// expose same-named methods but are not in scope here).
use ferriskey_core::domain::organization::ports::{
    AssignMemberRoleInput, ListMemberRolesInput, OrganizationId, OrganizationMemberRoleService,
    RevokeMemberRoleInput,
};
use ferriskey_core::domain::role::entities::Role;
use uuid::Uuid;

use crate::application::http::organization::validators::AssignMemberRoleValidator;
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    get,
    path = "/{organization_id}/members/{user_id}/roles",
    tag = "organization",
    summary = "List roles assigned to an organization member",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID of the member"),
    ),
    responses(
        (status = 200, description = "Roles scoped to the member", body = Vec<Role>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or member not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_member_roles(
    Path((realm_name, organization_id, user_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<Role>>, ApiError> {
    state
        .service
        .list_roles(
            identity,
            ListMemberRolesInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                user_id,
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{organization_id}/members/{user_id}/roles",
    tag = "organization",
    summary = "Assign a role to an organization member",
    request_body = AssignMemberRoleValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID of the member"),
    ),
    responses(
        (status = 204, description = "Role assigned"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or member not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn assign_member_role(
    Path((realm_name, organization_id, user_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<AssignMemberRoleValidator>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .assign_role(
            identity,
            AssignMemberRoleInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                user_id,
                role_id: payload.role_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/{organization_id}/members/{user_id}/roles/{role_id}",
    tag = "organization",
    summary = "Revoke a role from an organization member",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID of the member"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 204, description = "Role revoked"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or member not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn revoke_member_role(
    Path((realm_name, organization_id, user_id, role_id)): Path<(String, Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .revoke_role(
            identity,
            RevokeMemberRoleInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                user_id,
                role_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}
