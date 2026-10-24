use std::collections::HashSet;

use uuid::Uuid;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_domain::realm::{Realm, RealmId};
use ferriskey_domain::role::entities::Role;

use crate::entities::{
    AddGroupMemberInput, AddOrganizationMemberInput, AssignGroupRoleInput, AssignMemberRoleInput,
    CreateGroupInput, CreateGroupParams, CreateOrganizationInput, CreateOrganizationParams,
    DeleteGroupAttributeInput, DeleteGroupInput, DeleteOrganizationAttributeInput,
    DeleteOrganizationInput, GetGroupInput, GetOrganizationInput, Group, GroupAttribute,
    GroupFilter, GroupId, GroupListItem, GroupMember, GroupMemberDetail, GroupMemberFilter,
    GroupMemberSortField, GroupRoleMapping, GroupSortField, ListGroupAttributesInput,
    ListGroupMembersInput, ListGroupRolesInput, ListGroupsInput, ListMemberRolesInput,
    ListOrganizationAttributesInput, ListOrganizationMembersInput, ListUserOrganizationsInput,
    Organization, OrganizationAttribute, OrganizationFilter, OrganizationId, OrganizationMember,
    OrganizationMemberFilter, OrganizationMemberSortField, OrganizationSortField,
    RemoveGroupMemberInput, RemoveOrganizationMemberInput, RevokeGroupRoleInput,
    RevokeMemberRoleInput, UpdateGroupInput, UpdateGroupParams, UpdateOrganizationInput,
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
    ) -> impl Future<Output = Result<Option<Unscoped<Organization>>, CoreError>> + Send;

    fn get_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<OrganizationFilter, OrganizationSortField>,
    ) -> impl Future<Output = Result<Page<Organization>, CoreError>> + Send;

    fn update_organization(
        &self,
        organization: &Scoped<Organization>,
        params: UpdateOrganizationParams,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn delete_organization(
        &self,
        organization: &Scoped<Organization>,
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

    fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<OrganizationMemberFilter, OrganizationMemberSortField>,
    ) -> impl Future<Output = Result<Page<OrganizationMember>, CoreError>> + Send;

    fn list_organizations_for_user(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;

    fn get_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<OrganizationMember>, CoreError>> + Send;
}

/// Persistence for member→role mappings (roles scoped to an organization membership).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationMemberRoleRepository: Send + Sync {
    fn assign_role(
        &self,
        organization_member_id: Uuid,
        role: &Scoped<Role>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn revoke_role(
        &self,
        organization_member_id: Uuid,
        role: &Scoped<Role>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_role_ids(
        &self,
        organization_member_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;
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
        realm_name: String,
        request: PageRequest<OrganizationFilter, OrganizationSortField>,
    ) -> impl Future<Output = Result<Page<Organization>, CoreError>> + Send;

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
        request: PageRequest<OrganizationMemberFilter, OrganizationMemberSortField>,
    ) -> impl Future<Output = Result<Page<OrganizationMember>, CoreError>> + Send;

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

    fn list_groups_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Group>, CoreError>> + Send;

    fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<GroupFilter, GroupSortField>,
    ) -> impl Future<Output = Result<Page<GroupListItem>, CoreError>> + Send;

    fn update_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, CoreError>> + Send;

    fn delete_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
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

    fn list(
        &self,
        group: &Group,
        request: &PageRequest<GroupMemberFilter, GroupMemberSortField>,
    ) -> impl Future<Output = Result<Page<GroupMemberDetail>, CoreError>> + Send;

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
        role: &Scoped<Role>,
    ) -> impl Future<Output = Result<GroupRoleMapping, CoreError>> + Send;

    fn revoke_role(
        &self,
        group_id: GroupId,
        role: &Scoped<Role>,
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenGroupMembership {
    pub effective_groups: Vec<Group>,
    pub direct_group_ids: HashSet<Uuid>,
    pub group_role_ids_by_org: Vec<(Uuid, Uuid)>,
    pub member_role_ids_by_org: Vec<(Uuid, Uuid)>,
}

impl TokenGroupMembership {
    pub fn effective_role_ids(&self) -> Vec<Uuid> {
        let mut seen = HashSet::new();
        self.group_role_ids_by_org
            .iter()
            .map(|(_, role_id)| *role_id)
            .filter(|role_id| seen.insert(*role_id))
            .collect()
    }
}

/// Read model powering token issuance: resolves a user's *effective* (recursive) group
/// membership and inherited roles. Implemented with a recursive CTE.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait GroupTokenRepository: Send + Sync {
    /// Distinct role ids inherited from the user's effective (recursive) groups. Callers resolve
    /// these to full `Role`s (with client) via `UserRoleRepository::get_roles_by_ids`.
    fn list_effective_role_ids_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;

    /// Everything token issuance needs about a user's groups and organisation roles, in two
    /// queries: the effective (recursive) groups with the direct ones flagged, the roles
    /// inherited through those groups tagged with their organisation, and the roles assigned
    /// directly to the user's organisation memberships.
    fn token_membership(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<TokenGroupMembership, CoreError>> + Send;

    /// Same as `token_membership` for the service account of `client_id`, so it can run
    /// before the service account user is known.
    fn token_membership_of_service_account(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<TokenGroupMembership, CoreError>> + Send;
}

/// Service trait for member-scoped role business logic.
pub trait OrganizationMemberRoleService: Send + Sync {
    fn assign_role(
        &self,
        identity: Identity,
        input: AssignMemberRoleInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeMemberRoleInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Roles assigned directly to the member (resolved to full `Role`s).
    fn list_roles(
        &self,
        identity: Identity,
        input: ListMemberRolesInput,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
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

    fn list_groups(
        &self,
        identity: Identity,
        input: ListGroupsInput,
        request: PageRequest<GroupFilter, GroupSortField>,
    ) -> impl Future<Output = Result<Page<GroupListItem>, CoreError>> + Send;

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
        request: PageRequest<GroupMemberFilter, GroupMemberSortField>,
    ) -> impl Future<Output = Result<Page<GroupMemberDetail>, CoreError>> + Send;

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
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_organization(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_organization(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_organization(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_members(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
