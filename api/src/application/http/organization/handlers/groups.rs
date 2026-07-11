use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::organization::ports::{
    AddGroupMemberInput, AssignGroupRoleInput, CreateGroupInput, DeleteGroupAttributeInput,
    DeleteGroupInput, GetGroupInput, Group, GroupAttribute, GroupId, GroupMember, GroupMemberPage,
    GroupNode, GroupService, ListGroupAttributesInput, ListGroupMembersInput, ListGroupRolesInput,
    ListGroupsInput, OrganizationId, RemoveGroupMemberInput, RevokeGroupRoleInput,
    UpdateGroupInput, UpsertGroupAttributeInput,
};
use ferriskey_core::domain::role::entities::Role;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListGroupMembersQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    /// Case-insensitive filter on username/email.
    pub search: Option<String>,
}

use crate::application::http::organization::validators::{
    AddGroupMemberValidator, AssignGroupRoleValidator, CreateGroupValidator, UpdateGroupValidator,
    UpsertAttributeValidator,
};
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    get,
    path = "/{organization_id}/groups",
    tag = "organization",
    summary = "List an organization's groups as a tree",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Group tree", body = Vec<GroupNode>),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_groups(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<GroupNode>>, ApiError> {
    state
        .service
        .list_groups(
            identity,
            ListGroupsInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{organization_id}/groups",
    tag = "organization",
    summary = "Create a group",
    request_body = CreateGroupValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 201, description = "Group created", body = Group),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or parent group not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn create_group(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateGroupValidator>,
) -> Result<Response<Group>, ApiError> {
    state
        .service
        .create_group(
            identity,
            CreateGroupInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                parent_group_id: payload.parent_group_id.map(GroupId::new),
                name: payload.name,
                description: payload.description,
            },
        )
        .await
        .map(Response::Created)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/{organization_id}/groups/{group_id}",
    tag = "organization",
    summary = "Get a group",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "Group", body = Group),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_group(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Group>, ApiError> {
    state
        .service
        .get_group(
            identity,
            GetGroupInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    put,
    path = "/{organization_id}/groups/{group_id}",
    tag = "organization",
    summary = "Update a group",
    request_body = UpdateGroupValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "Group updated", body = Group),
        (status = 400, description = "Invalid parent (cycle)", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_group(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateGroupValidator>,
) -> Result<Response<Group>, ApiError> {
    state
        .service
        .update_group(
            identity,
            UpdateGroupInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                name: payload.name,
                description: payload.description,
                parent_group_id: payload.parent_group_id.map(|id| Some(GroupId::new(id))),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/{organization_id}/groups/{group_id}",
    tag = "organization",
    summary = "Delete a group (and its sub-groups)",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 204, description = "Group deleted"),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_group(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .delete_group(
            identity,
            DeleteGroupInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/{organization_id}/groups/{group_id}/members",
    tag = "organization",
    summary = "List group members",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    params(ListGroupMembersQuery),
    responses(
        (status = 200, description = "Members page", body = GroupMemberPage),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_group_members(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListGroupMembersQuery>,
) -> Result<Response<GroupMemberPage>, ApiError> {
    state
        .service
        .list_members(
            identity,
            ListGroupMembersInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                limit: query.limit,
                offset: query.offset,
                search: query.search,
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{organization_id}/groups/{group_id}/members",
    tag = "organization",
    summary = "Add a member to a group",
    request_body = AddGroupMemberValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 201, description = "Member added", body = GroupMember),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 409, description = "Already a member", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn add_group_member(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<AddGroupMemberValidator>,
) -> Result<Response<GroupMember>, ApiError> {
    state
        .service
        .add_member(
            identity,
            AddGroupMemberInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                user_id: payload.user_id,
            },
        )
        .await
        .map(Response::Created)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/{organization_id}/groups/{group_id}/members/{user_id}",
    tag = "organization",
    summary = "Remove a member from a group",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn remove_group_member(
    Path((realm_name, organization_id, group_id, user_id)): Path<(String, Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .remove_member(
            identity,
            RemoveGroupMemberInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                user_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/{organization_id}/groups/{group_id}/roles",
    tag = "organization",
    summary = "List roles assigned to a group",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "Roles", body = Vec<Role>),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_group_roles(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<Role>>, ApiError> {
    state
        .service
        .list_roles(
            identity,
            ListGroupRolesInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{organization_id}/groups/{group_id}/roles",
    tag = "organization",
    summary = "Assign a role to a group",
    request_body = AssignGroupRoleValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 204, description = "Role assigned"),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn assign_group_role(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<AssignGroupRoleValidator>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .assign_role(
            identity,
            AssignGroupRoleInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                role_id: payload.role_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/{organization_id}/groups/{group_id}/roles/{role_id}",
    tag = "organization",
    summary = "Revoke a role from a group",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("role_id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 204, description = "Role revoked"),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn revoke_group_role(
    Path((realm_name, organization_id, group_id, role_id)): Path<(String, Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .revoke_role(
            identity,
            RevokeGroupRoleInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                role_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

#[utoipa::path(
    get,
    path = "/{organization_id}/groups/{group_id}/attributes",
    tag = "organization",
    summary = "List group attributes",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "Attributes", body = Vec<GroupAttribute>),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_group_attributes(
    Path((realm_name, organization_id, group_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<GroupAttribute>>, ApiError> {
    state
        .service
        .list_attributes(
            identity,
            ListGroupAttributesInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    put,
    path = "/{organization_id}/groups/{group_id}/attributes/{key}",
    tag = "organization",
    summary = "Create or update a group attribute",
    request_body = UpsertAttributeValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("key" = String, Path, description = "Attribute key"),
    ),
    responses(
        (status = 200, description = "Attribute upserted", body = GroupAttribute),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn upsert_group_attribute(
    Path((realm_name, organization_id, group_id, key)): Path<(String, Uuid, Uuid, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpsertAttributeValidator>,
) -> Result<Response<GroupAttribute>, ApiError> {
    state
        .service
        .upsert_attribute(
            identity,
            UpsertGroupAttributeInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                key,
                value: payload.value,
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

#[utoipa::path(
    delete,
    path = "/{organization_id}/groups/{group_id}/attributes/{key}",
    tag = "organization",
    summary = "Delete a group attribute",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("key" = String, Path, description = "Attribute key"),
    ),
    responses(
        (status = 204, description = "Attribute deleted"),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_group_attribute(
    Path((realm_name, organization_id, group_id, key)): Path<(String, Uuid, Uuid, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .delete_attribute(
            identity,
            DeleteGroupAttributeInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                group_id: GroupId::new(group_id),
                key,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}
