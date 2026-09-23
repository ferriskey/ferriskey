use std::collections::HashMap;
use std::sync::Arc;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::{RealmScope, Scoped};
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::role::ports::RoleRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};
use tracing::warn;
use uuid::Uuid;

use crate::{
    AddGroupMemberInput, AssignGroupRoleInput, CreateGroupInput, CreateGroupParams,
    DeleteGroupAttributeInput, DeleteGroupInput, GetGroupInput, Group, GroupAttribute,
    GroupAttributeRepository, GroupConfig, GroupId, GroupMember, GroupMemberPage,
    GroupMemberRepository, GroupNode, GroupRepository, GroupRoleRepository, GroupService,
    ListGroupAttributesInput, ListGroupMembersInput, ListGroupRolesInput, ListGroupsInput,
    Organization, OrganizationId, OrganizationPolicy, OrganizationRepository,
    RemoveGroupMemberInput, RevokeGroupRoleInput, UpdateGroupInput, UpdateGroupParams,
    UpsertGroupAttributeInput, validate_membership_realms,
};

#[derive(Clone, Debug)]
pub struct GroupServiceImpl<R, U, C, UR, RO, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
    OR: OrganizationRepository,
    GR: GroupRepository,
    GMR: GroupMemberRepository,
    GRR: GroupRoleRepository,
    GAR: GroupAttributeRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) organization_repository: Arc<OR>,
    pub(crate) group_repository: Arc<GR>,
    pub(crate) group_member_repository: Arc<GMR>,
    pub(crate) group_role_repository: Arc<GRR>,
    pub(crate) group_attribute_repository: Arc<GAR>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, RO, OR, GR, GMR, GRR, GAR>
    GroupServiceImpl<R, U, C, UR, RO, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
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
        role_repository: Arc<RO>,
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
            role_repository,
            organization_repository,
            group_repository,
            group_member_repository,
            group_role_repository,
            group_attribute_repository,
            policy,
        }
    }

    async fn load_organization_in_realm(
        &self,
        realm_name: &str,
        organization_id: OrganizationId,
    ) -> Result<(RealmScope, Scoped<Organization>), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), realm_name).await?;

        let organization = self
            .organization_repository
            .get_organization_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?
            .in_realm(&scope)?;

        Ok((scope, organization))
    }

    async fn load_group_of_organization(
        &self,
        group_id: GroupId,
        organization: &Scoped<Organization>,
    ) -> Result<Group, CoreError> {
        let group = self
            .group_repository
            .get_group_by_id(group_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if group.organization_id != organization.get().id {
            warn!(
                group_id = %group_id,
                group_organization_id = %group.organization_id,
                request_organization_id = %organization.get().id,
                "Refused access to a group of another organization"
            );
            return Err(CoreError::NotFound);
        }

        Ok(group)
    }

    async fn load_member_of_group(
        &self,
        group: &Group,
        user_id: Uuid,
    ) -> Result<GroupMember, CoreError> {
        self.group_member_repository
            .get_member(group.id, user_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn load_role_in_realm(
        &self,
        role_id: Uuid,
        scope: &RealmScope,
    ) -> Result<Scoped<Role>, CoreError> {
        self.role_repository
            .get_by_id(role_id)
            .await?
            .ok_or_else(|| {
                warn!(role_id = %role_id, "Role not found");
                CoreError::NotFound
            })?
            .in_realm(scope)
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

impl<R, U, C, UR, RO, OR, GR, GMR, GRR, GAR> GroupService
    for GroupServiceImpl<R, U, C, UR, RO, OR, GR, GMR, GRR, GAR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    RO: RoleRepository,
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
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage groups",
        )?;

        // A provided parent must belong to the same organization.
        if let Some(parent_id) = input.parent_group_id {
            self.load_group_of_organization(parent_id, &org).await?;
        }

        let group = Group::new(GroupConfig {
            organization_id: org.get().id,
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
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_view_organization(&identity, scope.realm())
                .await,
            "insufficient permissions to view groups",
        )?;

        self.load_group_of_organization(input.group_id, &org).await
    }

    async fn list_groups(
        &self,
        identity: Identity,
        input: ListGroupsInput,
    ) -> Result<Vec<GroupNode>, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_view_organization(&identity, scope.realm())
                .await,
            "insufficient permissions to view groups",
        )?;

        let flat = self
            .group_repository
            .list_groups_by_organization(org.get().id)
            .await?;

        Ok(Self::build_tree(flat))
    }

    async fn update_group(
        &self,
        identity: Identity,
        input: UpdateGroupInput,
    ) -> Result<Group, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage groups",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;

        if let Some(Some(parent_id)) = input.parent_group_id {
            self.validate_parent(org.get().id, group.id, parent_id)
                .await?;
        }

        self.group_repository
            .update_group(
                org.get().id,
                group.id,
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
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage groups",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        self.group_repository
            .delete_group(org.get().id, group.id)
            .await
    }

    async fn add_member(
        &self,
        identity: Identity,
        input: AddGroupMemberInput,
    ) -> Result<GroupMember, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group members",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;

        let user = self
            .user_repository
            .get_by_id(input.user_id)
            .await?
            .in_realm(&scope)
            .map_err(|_| CoreError::Invalid)?;
        validate_membership_realms(org.get().realm_id, user.get().realm_id)
            .map_err(|_| CoreError::Invalid)?;

        if self
            .group_member_repository
            .get_member(group.id, input.user_id)
            .await?
            .is_some()
        {
            return Err(CoreError::AlreadyExists);
        }

        self.group_member_repository
            .add_member(group.id, input.user_id)
            .await
    }

    async fn remove_member(
        &self,
        identity: Identity,
        input: RemoveGroupMemberInput,
    ) -> Result<(), CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group members",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        let member = self.load_member_of_group(&group, input.user_id).await?;
        self.group_member_repository
            .remove_member(group.id, member.user_id)
            .await
    }

    async fn list_members(
        &self,
        identity: Identity,
        input: ListGroupMembersInput,
    ) -> Result<GroupMemberPage, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_view_organization(&identity, scope.realm())
                .await,
            "insufficient permissions to view group members",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;

        // Clamp pagination to sane bounds (default page of 50, hard max of 200).
        let limit = input.limit.unwrap_or(50).clamp(1, 200);
        let offset = input.offset.unwrap_or(0);

        let data = self
            .group_member_repository
            .list_members(group.id, limit, offset, input.search.clone())
            .await?;
        let total = self
            .group_member_repository
            .count_members(group.id, input.search)
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
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group roles",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        let role = self.load_role_in_realm(input.role_id, &scope).await?;
        self.group_role_repository
            .assign_role(group.id, &role)
            .await?;

        Ok(())
    }

    async fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeGroupRoleInput,
    ) -> Result<(), CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group roles",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        let role = self.load_role_in_realm(input.role_id, &scope).await?;
        self.group_role_repository
            .revoke_role(group.id, &role)
            .await
    }

    async fn list_roles(
        &self,
        identity: Identity,
        input: ListGroupRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_view_organization(&identity, scope.realm())
                .await,
            "insufficient permissions to view group roles",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        let role_ids = self.group_role_repository.list_role_ids(group.id).await?;

        self.user_role_repository.get_roles_by_ids(role_ids).await
    }

    async fn list_attributes(
        &self,
        identity: Identity,
        input: ListGroupAttributesInput,
    ) -> Result<Vec<GroupAttribute>, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_view_organization(&identity, scope.realm())
                .await,
            "insufficient permissions to view group attributes",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        self.group_attribute_repository
            .list_attributes(group.id)
            .await
    }

    async fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertGroupAttributeInput,
    ) -> Result<GroupAttribute, CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group attributes",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;

        // Validate key/value using the domain constructor before persisting.
        let attribute = GroupAttribute::new(group.id, input.key, input.value)
            .map_err(|_| CoreError::Invalid)?;

        self.group_attribute_repository
            .upsert_attribute(group.id, attribute.key, attribute.value)
            .await
    }

    async fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteGroupAttributeInput,
    ) -> Result<(), CoreError> {
        let (scope, org) = self
            .load_organization_in_realm(&input.realm_name, input.organization_id)
            .await?;
        ensure_policy(
            self.policy
                .can_manage_members(&identity, scope.realm())
                .await,
            "insufficient permissions to manage group attributes",
        )?;

        let group = self
            .load_group_of_organization(input.group_id, &org)
            .await?;
        self.group_attribute_repository
            .delete_attribute(group.id, &input.key)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::Realm;
    use ferriskey_domain::realm::scope::Unscoped;
    use ferriskey_domain::realm::{RealmId, ports::MockRealmRepository};
    use ferriskey_domain::role::ports::MockRoleRepository;
    use ferriskey_domain::user::entities::User;
    use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository};

    use crate::{
        MockGroupAttributeRepository, MockGroupMemberRepository, MockGroupRepository,
        MockGroupRoleRepository, MockOrganizationRepository,
    };

    use super::*;

    const ATTACKER_REALM: &str = "attacker-realm";
    const VICTIM_REALM: &str = "victim-realm";

    fn make_realm(id: RealmId, name: &str) -> Realm {
        Realm {
            id,
            name: name.to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            client_id: None,
            username: "admin".to_string(),
            firstname: Some("Admin".to_string()),
            lastname: Some("User".to_string()),
            email: Some("admin@test.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
            locale: None,
        }
    }

    fn make_role_with_permission(realm_id: RealmId, permission: &str) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![permission.to_string()],
            realm_id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_org(realm_id: RealmId) -> Organization {
        Organization {
            id: OrganizationId::new(Uuid::new_v4()),
            realm_id,
            name: "Test Org".to_string(),
            alias: "test-org".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_group(organization_id: OrganizationId) -> Group {
        Group {
            id: GroupId::new(Uuid::new_v4()),
            organization_id,
            parent_group_id: None,
            name: "Engineering".to_string(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    type TestService = GroupServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockRoleRepository,
        MockOrganizationRepository,
        MockGroupRepository,
        MockGroupMemberRepository,
        MockGroupRoleRepository,
        MockGroupAttributeRepository,
    >;

    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        org_repo: MockOrganizationRepository,
        group_repo: MockGroupRepository,
        group_member_repo: MockGroupMemberRepository,
    ) -> TestService {
        build_service_with_roles(
            realm_repo,
            user_repo,
            user_role_repo,
            MockRoleRepository::new(),
            org_repo,
            group_repo,
            group_member_repo,
            MockGroupRoleRepository::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_service_with_roles(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        role_repo: MockRoleRepository,
        org_repo: MockOrganizationRepository,
        group_repo: MockGroupRepository,
        group_member_repo: MockGroupMemberRepository,
        group_role_repo: MockGroupRoleRepository,
    ) -> TestService {
        let user_arc = Arc::new(user_repo);
        let user_role_arc = Arc::new(user_role_repo);
        let policy = Arc::new(FerriskeyPolicy::new(
            user_arc.clone(),
            Arc::new(MockClientRepository::new()),
            user_role_arc.clone(),
        ));

        GroupServiceImpl::new(
            Arc::new(realm_repo),
            user_arc,
            user_role_arc,
            Arc::new(role_repo),
            Arc::new(org_repo),
            Arc::new(group_repo),
            Arc::new(group_member_repo),
            Arc::new(group_role_repo),
            Arc::new(MockGroupAttributeRepository::new()),
            policy,
        )
    }

    /// Repositories that resolve `victim-realm` and answer the policy lookup for an attacker
    /// who is a genuine admin of `attacker-realm`. The denial under test must therefore come
    /// from the realm gate, not from missing permissions.
    fn cross_realm_actor(
        victim_realm_id: RealmId,
        permission: &'static str,
    ) -> (Identity, MockRealmRepository, MockUserRoleRepository) {
        let attacker_realm = make_realm(RealmId::new(Uuid::new_v4()), ATTACKER_REALM);
        let attacker_realm_id = attacker_realm.id;
        let identity = Identity::User(make_user(&attacker_realm));

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(victim_realm_id, VICTIM_REALM);
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(attacker_realm_id, permission);
            Box::pin(async move { Ok(vec![role]) })
        });

        (identity, realm_repo, user_role_repo)
    }

    fn org_repo_returning(org: Organization) -> MockOrganizationRepository {
        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .returning(move |_| {
                let o = org.clone();
                Box::pin(async move { Ok(Some(Unscoped::new(o))) })
            });
        org_repo
    }

    #[tokio::test]
    async fn create_group_denies_actor_from_another_realm() {
        let victim_realm_id = RealmId::new(Uuid::new_v4());
        let (identity, realm_repo, user_role_repo) =
            cross_realm_actor(victim_realm_id, "manage_users");
        let org = make_org(victim_realm_id);
        let org_id = org.id;

        // Permissive repository: if the policy leaks, the group is really created.
        let mut group_repo = MockGroupRepository::new();
        group_repo.expect_create_group().returning(move |_| {
            let g = make_group(org_id);
            Box::pin(async move { Ok(g) })
        });

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            org_repo_returning(org),
            group_repo,
            MockGroupMemberRepository::new(),
        );

        let result = service
            .create_group(
                identity,
                CreateGroupInput {
                    realm_name: VICTIM_REALM.to_string(),
                    organization_id: org_id,
                    parent_group_id: None,
                    name: "Pwned".to_string(),
                    description: None,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "an admin of another realm must not create groups here, got {result:?}"
        );
    }

    #[tokio::test]
    async fn list_groups_denies_actor_from_another_realm() {
        let victim_realm_id = RealmId::new(Uuid::new_v4());
        let (identity, realm_repo, user_role_repo) =
            cross_realm_actor(victim_realm_id, "view_users");
        let org = make_org(victim_realm_id);
        let org_id = org.id;

        let mut group_repo = MockGroupRepository::new();
        group_repo
            .expect_list_groups_by_organization()
            .returning(move |_| {
                let g = make_group(org_id);
                Box::pin(async move { Ok(vec![g]) })
            });

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            org_repo_returning(org),
            group_repo,
            MockGroupMemberRepository::new(),
        );

        let result = service
            .list_groups(
                identity,
                ListGroupsInput {
                    realm_name: VICTIM_REALM.to_string(),
                    organization_id: org_id,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "an actor of another realm must not enumerate groups here, got {result:?}"
        );
    }

    #[tokio::test]
    async fn add_member_denies_actor_from_another_realm() {
        let victim_realm_id = RealmId::new(Uuid::new_v4());
        let victim_realm = make_realm(victim_realm_id, VICTIM_REALM);
        let victim_user = make_user(&victim_realm);
        let victim_user_id = victim_user.id;

        let (identity, realm_repo, user_role_repo) =
            cross_realm_actor(victim_realm_id, "manage_users");
        let org = make_org(victim_realm_id);
        let org_id = org.id;
        let group = make_group(org_id);
        let group_id = group.id;

        let mut group_repo = MockGroupRepository::new();
        group_repo.expect_get_group_by_id().returning(move |_| {
            let g = group.clone();
            Box::pin(async move { Ok(Some(g)) })
        });

        // The user really belongs to the victim realm, so `validate_membership_realms` passes.
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = victim_user.clone();
            Box::pin(async move { Ok(Unscoped::new(u)) })
        });

        let mut group_member_repo = MockGroupMemberRepository::new();
        group_member_repo
            .expect_get_member()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        group_member_repo.expect_add_member().returning(|g, u| {
            let m = GroupMember::new(g, u);
            Box::pin(async move { Ok(m) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo_returning(org),
            group_repo,
            group_member_repo,
        );

        let result = service
            .add_member(
                identity,
                AddGroupMemberInput {
                    realm_name: VICTIM_REALM.to_string(),
                    organization_id: org_id,
                    group_id,
                    user_id: victim_user_id,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "an admin of another realm must not add members to this group, got {result:?}"
        );
    }

    #[tokio::test]
    async fn create_group_succeeds_for_same_realm_admin() {
        // Counterpart to the cross-realm cases: the realm gate must not deny the legitimate
        // administrator of the organization's own realm.
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let identity = Identity::User(make_user(&realm));
        let org = make_org(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let mut group_repo = MockGroupRepository::new();
        group_repo.expect_create_group().returning(move |_| {
            let g = make_group(org_id);
            Box::pin(async move { Ok(g) })
        });

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            org_repo_returning(org),
            group_repo,
            MockGroupMemberRepository::new(),
        );

        let result = service
            .create_group(
                identity,
                CreateGroupInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    parent_group_id: None,
                    name: "Engineering".to_string(),
                    description: None,
                },
            )
            .await;

        assert!(
            result.is_ok(),
            "the organization's own realm admin must be allowed, got {result:?}"
        );
    }

    #[tokio::test]
    async fn assign_role_refuses_a_role_from_another_realm() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let identity = Identity::User(make_user(&realm));
        let org = make_org(realm_id);
        let org_id = org.id;
        let group = make_group(org_id);
        let group_id = group.id;
        let foreign_role = make_role_with_permission(RealmId::new(Uuid::new_v4()), "manage_realm");
        let foreign_role_id = foreign_role.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let mut group_repo = MockGroupRepository::new();
        group_repo.expect_get_group_by_id().returning(move |_| {
            let g = group.clone();
            Box::pin(async move { Ok(Some(g)) })
        });

        let mut role_repo = MockRoleRepository::new();
        role_repo.expect_get_by_id().returning(move |_| {
            let r = foreign_role.clone();
            Box::pin(async move { Ok(Some(Unscoped::new(r))) })
        });

        let mut group_role_repo = MockGroupRoleRepository::new();
        group_role_repo.expect_assign_role().times(0);

        let service = build_service_with_roles(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            role_repo,
            org_repo_returning(org),
            group_repo,
            MockGroupMemberRepository::new(),
            group_role_repo,
        );

        let result = service
            .assign_role(
                identity,
                AssignGroupRoleInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    group_id,
                    role_id: foreign_role_id,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::NotFound)),
            "a role of another realm must not be attachable to this group, got {result:?}"
        );
    }
}
