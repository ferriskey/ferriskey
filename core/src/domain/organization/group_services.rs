use std::collections::HashMap;
use std::sync::Arc;

use ferriskey_organization::{GroupConfig, validate_membership_realms};

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    organization::ports::{
        AddGroupMemberInput, AssignGroupRoleInput, CreateGroupInput, CreateGroupParams,
        DeleteGroupAttributeInput, DeleteGroupInput, GetGroupInput, Group, GroupAttribute,
        GroupAttributeRepository, GroupId, GroupMember, GroupMemberPage, GroupMemberRepository,
        GroupNode, GroupRepository, GroupRoleRepository, GroupService, ListGroupAttributesInput,
        ListGroupMembersInput, ListGroupRolesInput, ListGroupsInput, Organization, OrganizationId,
        OrganizationPolicy, OrganizationRepository, RemoveGroupMemberInput, RevokeGroupRoleInput,
        UpdateGroupInput, UpdateGroupParams, UpsertGroupAttributeInput,
    },
    realm::ports::RealmRepository,
    role::entities::Role,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct GroupServiceImpl<R, U, C, UR, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    GR: GroupRepository,
    GMR: GroupMemberRepository,
    GRR: GroupRoleRepository,
    GAR: GroupAttributeRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) organization_repository: Arc<OR>,
    pub(crate) group_repository: Arc<GR>,
    pub(crate) group_member_repository: Arc<GMR>,
    pub(crate) group_role_repository: Arc<GRR>,
    pub(crate) group_attribute_repository: Arc<GAR>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, OR, GR, GMR, GRR, GAR> GroupServiceImpl<R, U, C, UR, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    GR: GroupRepository,
    GMR: GroupMemberRepository,
    GRR: GroupRoleRepository,
    GAR: GroupAttributeRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        user_role_repository: Arc<UR>,
        organization_repository: Arc<OR>,
        group_repository: Arc<GR>,
        group_member_repository: Arc<GMR>,
        group_role_repository: Arc<GRR>,
        group_attribute_repository: Arc<GAR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            user_role_repository,
            organization_repository,
            group_repository,
            group_member_repository,
            group_role_repository,
            group_attribute_repository,
            policy,
        }
    }

    async fn get_org(
        &self,
        realm_name: String,
        organization_id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let org = self
            .organization_repository
            .get_organization_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if org.realm_id != realm.id {
            return Err(CoreError::NotFound);
        }

        Ok(org)
    }

    /// Load a group and assert it belongs to `organization_id`.
    async fn get_group_in_org(
        &self,
        organization_id: OrganizationId,
        group_id: GroupId,
    ) -> Result<Group, CoreError> {
        let group = self
            .group_repository
            .get_group_by_id(group_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if group.organization_id != organization_id {
            return Err(CoreError::NotFound);
        }

        Ok(group)
    }

    /// Reject a parent assignment that would create a cycle (parent is the group itself
    /// or one of its descendants) or that points outside the organization.
    async fn validate_parent(
        &self,
        organization_id: OrganizationId,
        group_id: GroupId,
        parent_group_id: GroupId,
    ) -> Result<(), CoreError> {
        if parent_group_id == group_id {
            return Err(CoreError::Invalid);
        }

        let flat = self
            .group_repository
            .list_groups_by_organization(organization_id)
            .await?;
        let parent_map: HashMap<GroupId, Option<GroupId>> =
            flat.iter().map(|g| (g.id, g.parent_group_id)).collect();

        if !parent_map.contains_key(&parent_group_id) {
            // Parent must exist within the same organization.
            return Err(CoreError::NotFound);
        }

        // Walk up from the proposed parent; if we reach `group_id`, it's a cycle.
        let mut cursor = Some(parent_group_id);
        while let Some(current) = cursor {
            if current == group_id {
                return Err(CoreError::Invalid);
            }
            cursor = parent_map.get(&current).copied().flatten();
        }

        Ok(())
    }

    fn build_tree(flat: Vec<Group>) -> Vec<GroupNode> {
        let mut children: HashMap<Option<GroupId>, Vec<Group>> = HashMap::new();
        for group in flat {
            children
                .entry(group.parent_group_id)
                .or_default()
                .push(group);
        }
        build_nodes(None, &children)
    }
}

fn build_nodes(
    parent: Option<GroupId>,
    children: &HashMap<Option<GroupId>, Vec<Group>>,
) -> Vec<GroupNode> {
    children
        .get(&parent)
        .map(|groups| {
            groups
                .iter()
                .map(|group| GroupNode {
                    group: group.clone(),
                    children: build_nodes(Some(group.id), children),
                })
                .collect()
        })
        .unwrap_or_default()
}

impl<R, U, C, UR, OR, GR, GMR, GRR, GAR> GroupService
    for GroupServiceImpl<R, U, C, UR, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    GR: GroupRepository,
    GMR: GroupMemberRepository,
    GRR: GroupRoleRepository,
    GAR: GroupAttributeRepository,
{
    async fn create_group(
        &self,
        identity: Identity,
        input: CreateGroupInput,
    ) -> Result<Group, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage groups",
        )?;

        // A provided parent must belong to the same organization.
        if let Some(parent_id) = input.parent_group_id {
            self.get_group_in_org(org.id, parent_id).await?;
        }

        let group = Group::new(GroupConfig {
            organization_id: org.id,
            parent_group_id: input.parent_group_id,
            name: input.name,
            description: input.description,
        })
        .map_err(|_| CoreError::Invalid)?;

        self.group_repository
            .create_group(CreateGroupParams {
                organization_id: group.organization_id,
                parent_group_id: group.parent_group_id,
                name: group.name.clone(),
                description: group.description.clone(),
            })
            .await
    }

    async fn get_group(
        &self,
        identity: Identity,
        input: GetGroupInput,
    ) -> Result<Group, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view groups",
        )?;

        self.get_group_in_org(org.id, input.group_id).await
    }

    async fn list_groups(
        &self,
        identity: Identity,
        input: ListGroupsInput,
    ) -> Result<Vec<GroupNode>, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view groups",
        )?;

        let flat = self
            .group_repository
            .list_groups_by_organization(org.id)
            .await?;

        Ok(Self::build_tree(flat))
    }

    async fn update_group(
        &self,
        identity: Identity,
        input: UpdateGroupInput,
    ) -> Result<Group, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage groups",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;

        if let Some(Some(parent_id)) = input.parent_group_id {
            self.validate_parent(org.id, input.group_id, parent_id)
                .await?;
        }

        self.group_repository
            .update_group(
                input.group_id,
                UpdateGroupParams {
                    name: input.name,
                    description: input.description,
                    parent_group_id: input.parent_group_id,
                },
            )
            .await
    }

    async fn delete_group(
        &self,
        identity: Identity,
        input: DeleteGroupInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage groups",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_repository.delete_group(input.group_id).await
    }

    async fn add_member(
        &self,
        identity: Identity,
        input: AddGroupMemberInput,
    ) -> Result<GroupMember, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group members",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;

        let user = self.user_repository.get_by_id(input.user_id).await?;
        validate_membership_realms(org.realm_id, user.realm_id).map_err(|_| CoreError::Invalid)?;

        if self
            .group_member_repository
            .get_member(input.group_id, input.user_id)
            .await?
            .is_some()
        {
            return Err(CoreError::AlreadyExists);
        }

        self.group_member_repository
            .add_member(input.group_id, input.user_id)
            .await
    }

    async fn remove_member(
        &self,
        identity: Identity,
        input: RemoveGroupMemberInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group members",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_member_repository
            .remove_member(input.group_id, input.user_id)
            .await
    }

    async fn list_members(
        &self,
        identity: Identity,
        input: ListGroupMembersInput,
    ) -> Result<GroupMemberPage, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view group members",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;

        // Clamp pagination to sane bounds (default page of 50, hard max of 200).
        let limit = input.limit.unwrap_or(50).clamp(1, 200);
        let offset = input.offset.unwrap_or(0);

        let data = self
            .group_member_repository
            .list_members(input.group_id, limit, offset, input.search.clone())
            .await?;
        let total = self
            .group_member_repository
            .count_members(input.group_id, input.search)
            .await?;

        Ok(GroupMemberPage {
            data,
            total,
            limit,
            offset,
        })
    }

    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignGroupRoleInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group roles",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_role_repository
            .assign_role(input.group_id, input.role_id)
            .await?;

        Ok(())
    }

    async fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeGroupRoleInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group roles",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_role_repository
            .revoke_role(input.group_id, input.role_id)
            .await
    }

    async fn list_roles(
        &self,
        identity: Identity,
        input: ListGroupRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view group roles",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        let role_ids = self
            .group_role_repository
            .list_role_ids(input.group_id)
            .await?;

        self.user_role_repository.get_roles_by_ids(role_ids).await
    }

    async fn list_attributes(
        &self,
        identity: Identity,
        input: ListGroupAttributesInput,
    ) -> Result<Vec<GroupAttribute>, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view group attributes",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_attribute_repository
            .list_attributes(input.group_id)
            .await
    }

    async fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertGroupAttributeInput,
    ) -> Result<GroupAttribute, CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group attributes",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;

        // Validate key/value using the domain constructor before persisting.
        let attribute = GroupAttribute::new(input.group_id, input.key, input.value)
            .map_err(|_| CoreError::Invalid)?;

        self.group_attribute_repository
            .upsert_attribute(input.group_id, attribute.key, attribute.value)
            .await
    }

    async fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteGroupAttributeInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org(input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage group attributes",
        )?;

        self.get_group_in_org(org.id, input.group_id).await?;
        self.group_attribute_repository
            .delete_attribute(input.group_id, &input.key)
            .await
    }
}
