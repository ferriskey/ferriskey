use uuid::Uuid;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::role::entities::Role;

use crate::entities::{
    AddGroupMemberInput, AddOrganizationMemberInput, AssignGroupRoleInput, CreateGroupInput,
    CreateGroupParams, CreateOrganizationInput, CreateOrganizationParams,
    DeleteGroupAttributeInput, DeleteGroupInput, DeleteOrganizationAttributeInput,
    DeleteOrganizationInput, GetGroupInput, GetOrganizationInput, Group, GroupAttribute, GroupId,
    GroupMember, GroupMemberDetail, GroupMemberPage, GroupNode, GroupRoleMapping,
    ListGroupAttributesInput, ListGroupMembersInput, ListGroupRolesInput, ListGroupsInput,
    ListOrganizationAttributesInput, ListOrganizationMembersInput, ListOrganizationsInput,
    ListUserOrganizationsInput, Organization, OrganizationAttribute, OrganizationId,
    OrganizationMember, RemoveGroupMemberInput, RemoveOrganizationMemberInput,
    RevokeGroupRoleInput, UpdateGroupInput, UpdateGroupParams, UpdateOrganizationInput,
    UpdateOrganizationParams, UpsertGroupAttributeInput, UpsertOrganizationAttributeInput,
};

/// Repository trait for Organization persistence
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationRepository: Send + Sync {
    fn create_organization(
        &self,
        params: CreateOrganizationParams,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn get_organization_by_id(
        &self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    fn get_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    fn list_organizations_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Organization>, CoreError>> + Send;

    fn update_organization(
        &self,
        id: OrganizationId,
        params: UpdateOrganizationParams,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn delete_organization(
        &self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn exists_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

/// Repository trait for OrganizationAttribute persistence
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationAttributeRepository: Send + Sync {
    fn list_attributes(
        &self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<OrganizationAttribute>, CoreError>> + Send;

    fn upsert_attribute(
        &self,
        organization_id: OrganizationId,
        key: String,
        value: String,
    ) -> impl Future<Output = Result<OrganizationAttribute, CoreError>> + Send;

    fn delete_attribute(
        &self,
        organization_id: OrganizationId,
        key: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Repository trait for OrganizationMember persistence
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationMemberRepository: Send + Sync {
    fn add_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<OrganizationMember, CoreError>> + Send;

    fn remove_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_members(
        &self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;

    fn list_organizations_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;

    fn get_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<OrganizationMember>, CoreError>> + Send;
}

/// Service trait for Organization business logic
pub trait OrganizationService: Send + Sync {
    fn create_organization(
        &self,
        identity: Identity,
        input: CreateOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn get_organization(
        &self,
        identity: Identity,
        input: GetOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn list_organizations(
        &self,
        identity: Identity,
        input: ListOrganizationsInput,
    ) -> impl Future<Output = Result<Vec<Organization>, CoreError>> + Send;

    fn update_organization(
        &self,
        identity: Identity,
        input: UpdateOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn delete_organization(
        &self,
        identity: Identity,
        input: DeleteOrganizationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_attributes(
        &self,
        identity: Identity,
        input: ListOrganizationAttributesInput,
    ) -> impl Future<Output = Result<Vec<OrganizationAttribute>, CoreError>> + Send;

    fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertOrganizationAttributeInput,
    ) -> impl Future<Output = Result<OrganizationAttribute, CoreError>> + Send;

    fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteOrganizationAttributeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn add_member(
        &self,
        identity: Identity,
        input: AddOrganizationMemberInput,
    ) -> impl Future<Output = Result<OrganizationMember, CoreError>> + Send;

    fn remove_member(
        &self,
        identity: Identity,
        input: RemoveOrganizationMemberInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_members(
        &self,
        identity: Identity,
        input: ListOrganizationMembersInput,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;

    fn list_user_organizations(
        &self,
        identity: Identity,
        input: ListUserOrganizationsInput,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;
}

// ============================================================================
// Group repositories
// ============================================================================

/// Persistence for groups (the tree nodes themselves).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupRepository: Send + Sync {
    fn create_group(
        &self,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    fn get_group_by_id(
        &self,
        id: GroupId,
    ) -> impl Future<Output = Result<Option<Group>, CoreError>> + Send;

    /// Flat list of all groups in an organization (tree built in the service layer).
    fn list_groups_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Group>, CoreError>> + Send;

    fn update_group(
        &self,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    fn delete_group(&self, id: GroupId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Persistence for group memberships.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupMemberRepository: Send + Sync {
    fn add_member(
        &self,
        group_id: GroupId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<GroupMember, CoreError>> + Send;

    fn remove_member(
        &self,
        group_id: GroupId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Members of a group (enriched with user identity), paginated + optionally filtered.
    fn list_members(
        &self,
        group_id: GroupId,
        limit: u32,
        offset: u32,
        search: Option<String>,
    ) -> impl Future<Output = Result<Vec<GroupMemberDetail>, CoreError>> + Send;

    /// Total number of members matching `search` (for pagination totals).
    fn count_members(
        &self,
        group_id: GroupId,
        search: Option<String>,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;

    fn get_member(
        &self,
        group_id: GroupId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<GroupMember>, CoreError>> + Send;
}

/// Persistence for group→role mappings.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupRoleRepository: Send + Sync {
    fn assign_role(
        &self,
        group_id: GroupId,
        role_id: Uuid,
    ) -> impl Future<Output = Result<GroupRoleMapping, CoreError>> + Send;

    fn revoke_role(
        &self,
        group_id: GroupId,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_role_ids(
        &self,
        group_id: GroupId,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;
}

/// Persistence for group attributes.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupAttributeRepository: Send + Sync {
    fn list_attributes(
        &self,
        group_id: GroupId,
    ) -> impl Future<Output = Result<Vec<GroupAttribute>, CoreError>> + Send;

    fn upsert_attribute(
        &self,
        group_id: GroupId,
        key: String,
        value: String,
    ) -> impl Future<Output = Result<GroupAttribute, CoreError>> + Send;

    fn delete_attribute(
        &self,
        group_id: GroupId,
        key: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Read model powering token issuance: resolves a user's *effective* (recursive) group
/// membership and inherited roles via a single query each. Implemented with a recursive CTE.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupTokenRepository: Send + Sync {
    /// The groups a user belongs to plus all their ancestors (deduplicated).
    fn list_effective_groups_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Group>, CoreError>> + Send;

    /// Ids of the groups the user is a *direct* member of (no ancestor expansion). Used by the
    /// group-membership mapper in `direct` mode to distinguish direct memberships from inherited
    /// ancestors already present in the effective set.
    fn list_direct_group_ids_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;

    /// Distinct role ids inherited from the user's effective (recursive) groups. Callers resolve
    /// these to full `Role`s (with client) via `UserRoleRepository::get_roles_by_ids`.
    fn list_effective_role_ids_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;
}

/// Service trait for group business logic.
pub trait GroupService: Send + Sync {
    fn create_group(
        &self,
        identity: Identity,
        input: CreateGroupInput,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    fn get_group(
        &self,
        identity: Identity,
        input: GetGroupInput,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    /// Returns the organization's groups as a tree.
    fn list_groups(
        &self,
        identity: Identity,
        input: ListGroupsInput,
    ) -> impl Future<Output = Result<Vec<GroupNode>, CoreError>> + Send;

    fn update_group(
        &self,
        identity: Identity,
        input: UpdateGroupInput,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    fn delete_group(
        &self,
        identity: Identity,
        input: DeleteGroupInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn add_member(
        &self,
        identity: Identity,
        input: AddGroupMemberInput,
    ) -> impl Future<Output = Result<GroupMember, CoreError>> + Send;

    fn remove_member(
        &self,
        identity: Identity,
        input: RemoveGroupMemberInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_members(
        &self,
        identity: Identity,
        input: ListGroupMembersInput,
    ) -> impl Future<Output = Result<GroupMemberPage, CoreError>> + Send;

    fn assign_role(
        &self,
        identity: Identity,
        input: AssignGroupRoleInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeGroupRoleInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Roles directly assigned to a group (resolved to full `Role`s).
    fn list_roles(
        &self,
        identity: Identity,
        input: ListGroupRolesInput,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;

    fn list_attributes(
        &self,
        identity: Identity,
        input: ListGroupAttributesInput,
    ) -> impl Future<Output = Result<Vec<GroupAttribute>, CoreError>> + Send;

    fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertGroupAttributeInput,
    ) -> impl Future<Output = Result<GroupAttribute, CoreError>> + Send;

    fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteGroupAttributeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Policy trait for Organization authorization
pub trait OrganizationPolicy: Send + Sync {
    fn can_create_organization(
        &self,
        identity: &Identity,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_members(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
