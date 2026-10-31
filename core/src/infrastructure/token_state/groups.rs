use std::sync::Arc;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::scope::Scoped;
use ferriskey_domain::role::entities::Role;
use ferriskey_organization::{
    CreateGroupParams, Group, GroupFilter, GroupId, GroupListItem, GroupMember, GroupMemberDetail,
    GroupMemberFilter, GroupMemberRepository, GroupMemberSortField, GroupRepository,
    GroupRoleMapping, GroupRoleRepository, GroupSortField, GroupTokenRepository, Organization,
    OrganizationId, TokenGroupMembership, UpdateGroupParams,
};
use uuid::Uuid;

use super::state::TokenState;

#[derive(Debug, Clone)]
pub struct CachedGroupTokenRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedGroupTokenRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: GroupTokenRepository> GroupTokenRepository for CachedGroupTokenRepository<R> {
    async fn list_effective_role_ids_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, CoreError> {
        self.inner.list_effective_role_ids_for_user(user_id).await
    }

    async fn token_membership(&self, user_id: Uuid) -> Result<TokenGroupMembership, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().memberships,
                &user_id,
                |_| true,
                || self.inner.token_membership(user_id),
            )
            .await
    }

    async fn token_membership_of_service_account(
        &self,
        client_id: Uuid,
    ) -> Result<TokenGroupMembership, CoreError> {
        self.inner
            .token_membership_of_service_account(client_id)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingGroupRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingGroupRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: GroupRepository> GroupRepository for InvalidatingGroupRepository<R> {
    async fn create_group(&self, params: CreateGroupParams) -> Result<Group, CoreError> {
        self.inner.create_group(params).await
    }

    async fn get_group_by_id(&self, id: GroupId) -> Result<Option<Group>, CoreError> {
        self.inner.get_group_by_id(id).await
    }

    async fn list_groups_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Group>, CoreError> {
        self.inner
            .list_groups_by_organization(organization_id)
            .await
    }

    async fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<GroupFilter, GroupSortField>,
    ) -> Result<Page<GroupListItem>, CoreError> {
        self.inner.list(organization, request).await
    }

    async fn update_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, CoreError> {
        let updated = self.inner.update_group(organization_id, id, params).await;
        self.state.changed();
        updated
    }

    async fn delete_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
    ) -> Result<(), CoreError> {
        let deleted = self.inner.delete_group(organization_id, id).await;
        self.state.changed();
        deleted
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingGroupMemberRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingGroupMemberRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: GroupMemberRepository> GroupMemberRepository for InvalidatingGroupMemberRepository<R> {
    async fn add_member(&self, group_id: GroupId, user_id: Uuid) -> Result<GroupMember, CoreError> {
        let added = self.inner.add_member(group_id, user_id).await;
        self.state.changed();
        added
    }

    async fn remove_member(&self, group_id: GroupId, user_id: Uuid) -> Result<(), CoreError> {
        let removed = self.inner.remove_member(group_id, user_id).await;
        self.state.changed();
        removed
    }

    async fn list(
        &self,
        group: &Group,
        request: &PageRequest<GroupMemberFilter, GroupMemberSortField>,
    ) -> Result<Page<GroupMemberDetail>, CoreError> {
        self.inner.list(group, request).await
    }

    async fn get_member(
        &self,
        group_id: GroupId,
        user_id: Uuid,
    ) -> Result<Option<GroupMember>, CoreError> {
        self.inner.get_member(group_id, user_id).await
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingGroupRoleRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingGroupRoleRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: GroupRoleRepository> GroupRoleRepository for InvalidatingGroupRoleRepository<R> {
    async fn assign_role(
        &self,
        group_id: GroupId,
        role: &Scoped<Role>,
    ) -> Result<GroupRoleMapping, CoreError> {
        let assigned = self.inner.assign_role(group_id, role).await;
        self.state.changed();
        assigned
    }

    async fn revoke_role(&self, group_id: GroupId, role: &Scoped<Role>) -> Result<(), CoreError> {
        let revoked = self.inner.revoke_role(group_id, role).await;
        self.state.changed();
        revoked
    }

    async fn list_role_ids(&self, group_id: GroupId) -> Result<Vec<Uuid>, CoreError> {
        self.inner.list_role_ids(group_id).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_organization::{
        MockGroupMemberRepository, MockGroupRepository, MockGroupRoleRepository,
        MockGroupTokenRepository,
    };

    use super::*;
    use crate::infrastructure::token_state::test_support::{
        disabled_state, role, scoped, synced_state, unsynced_state,
    };

    fn group() -> Group {
        let now = Utc::now();
        Group {
            id: GroupId::new(Uuid::new_v4()),
            organization_id: OrganizationId::new(Uuid::new_v4()),
            parent_group_id: None,
            name: "staff".to_string(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn resolving_membership(calls: usize) -> MockGroupTokenRepository {
        let mut inner = MockGroupTokenRepository::new();
        inner
            .expect_token_membership()
            .times(calls)
            .returning(|_| Box::pin(async { Ok(TokenGroupMembership::default()) }));
        inner
    }

    #[tokio::test]
    async fn a_membership_read_twice_is_loaded_once() {
        let repository =
            CachedGroupTokenRepository::new(Arc::new(resolving_membership(1)), synced_state());
        let user_id = Uuid::new_v4();

        repository.token_membership(user_id).await.expect("first");
        repository.token_membership(user_id).await.expect("second");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_membership_read_goes_to_the_store() {
        let repository =
            CachedGroupTokenRepository::new(Arc::new(resolving_membership(2)), unsynced_state());
        let user_id = Uuid::new_v4();

        repository.token_membership(user_id).await.expect("first");
        repository.token_membership(user_id).await.expect("second");
    }

    #[tokio::test]
    async fn without_a_ttl_every_membership_read_goes_to_the_store() {
        let repository =
            CachedGroupTokenRepository::new(Arc::new(resolving_membership(2)), disabled_state());
        let user_id = Uuid::new_v4();

        repository.token_membership(user_id).await.expect("first");
        repository.token_membership(user_id).await.expect("second");
    }

    #[tokio::test]
    async fn a_group_move_makes_the_next_membership_read_go_to_the_store() {
        let state = synced_state();
        let tokens =
            CachedGroupTokenRepository::new(Arc::new(resolving_membership(2)), state.clone());
        let stored = group();
        let moved = stored.clone();
        let mut inner = MockGroupRepository::new();
        inner
            .expect_update_group()
            .times(1)
            .returning(move |_, _, _| {
                let group = moved.clone();
                Box::pin(async move { Ok(group) })
            });
        inner
            .expect_delete_group()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let groups = InvalidatingGroupRepository::new(Arc::new(inner), state.clone());
        let user_id = Uuid::new_v4();
        tokens.token_membership(user_id).await.expect("warm");

        groups
            .update_group(
                stored.organization_id,
                stored.id,
                UpdateGroupParams {
                    name: None,
                    description: None,
                    parent_group_id: Some(None),
                },
            )
            .await
            .expect("move");
        tokens.token_membership(user_id).await.expect("after move");
        groups
            .delete_group(stored.organization_id, stored.id)
            .await
            .expect("delete");
        assert_eq!(state.generation(), 2);
    }

    #[tokio::test]
    async fn every_group_member_write_announces_a_change() {
        let state = synced_state();
        let mut inner = MockGroupMemberRepository::new();
        inner.expect_add_member().returning(|group_id, user_id| {
            let member = GroupMember::new(group_id, user_id);
            Box::pin(async move { Ok(member) })
        });
        inner
            .expect_remove_member()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository = InvalidatingGroupMemberRepository::new(Arc::new(inner), state.clone());
        let group_id = GroupId::new(Uuid::new_v4());

        repository
            .add_member(group_id, Uuid::new_v4())
            .await
            .expect("add");
        assert_eq!(state.generation(), 1);
        repository
            .remove_member(group_id, Uuid::new_v4())
            .await
            .expect("remove");
        assert_eq!(state.generation(), 2);
    }

    #[tokio::test]
    async fn every_group_role_write_announces_a_change() {
        let state = synced_state();
        let mut inner = MockGroupRoleRepository::new();
        inner.expect_assign_role().returning(|group_id, role| {
            let mapping = GroupRoleMapping::new(group_id, role.get().id);
            Box::pin(async move { Ok(mapping) })
        });
        inner
            .expect_revoke_role()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository = InvalidatingGroupRoleRepository::new(Arc::new(inner), state.clone());
        let group_id = GroupId::new(Uuid::new_v4());
        let target = scoped(role());

        repository
            .assign_role(group_id, &target)
            .await
            .expect("assign");
        assert_eq!(state.generation(), 1);
        repository
            .revoke_role(group_id, &target)
            .await
            .expect("revoke");
        assert_eq!(state.generation(), 2);
    }
}
