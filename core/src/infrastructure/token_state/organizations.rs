use std::sync::Arc;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_domain::role::entities::Role;
use ferriskey_organization::{
    CreateOrganizationParams, Organization, OrganizationAttribute, OrganizationAttributeRepository,
    OrganizationFilter, OrganizationId, OrganizationMember, OrganizationMemberFilter,
    OrganizationMemberRepository, OrganizationMemberRoleRepository, OrganizationMemberSortField,
    OrganizationRepository, OrganizationSortField, UpdateOrganizationParams,
};
use uuid::Uuid;

use super::state::TokenState;

#[derive(Debug, Clone)]
pub struct CachedOrganizationRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedOrganizationRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: OrganizationRepository> OrganizationRepository for CachedOrganizationRepository<R> {
    async fn create_organization(
        &self,
        params: CreateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        self.inner.create_organization(params).await
    }

    async fn get_organization_by_id(
        &self,
        id: OrganizationId,
    ) -> Result<Option<Unscoped<Organization>>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().organizations,
                &id.as_uuid(),
                Option::is_some,
                || self.inner.get_organization_by_id(id),
            )
            .await
    }

    async fn get_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<Option<Organization>, CoreError> {
        self.inner
            .get_organization_by_realm_and_alias(realm_id, alias)
            .await
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<OrganizationFilter, OrganizationSortField>,
    ) -> Result<Page<Organization>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn update_organization(
        &self,
        organization: &Scoped<Organization>,
        params: UpdateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        let updated = self.inner.update_organization(organization, params).await;
        self.state.changed();
        updated
    }

    async fn delete_organization(
        &self,
        organization: &Scoped<Organization>,
    ) -> Result<(), CoreError> {
        let deleted = self.inner.delete_organization(organization).await;
        self.state.changed();
        deleted
    }

    async fn exists_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<bool, CoreError> {
        self.inner
            .exists_organization_by_realm_and_alias(realm_id, alias)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CachedOrganizationAttributeRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedOrganizationAttributeRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: OrganizationAttributeRepository> OrganizationAttributeRepository
    for CachedOrganizationAttributeRepository<R>
{
    async fn list_attributes(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationAttribute>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().organization_attributes,
                &organization_id.as_uuid(),
                |_| true,
                || self.inner.list_attributes(organization_id),
            )
            .await
    }

    async fn upsert_attribute(
        &self,
        organization_id: OrganizationId,
        key: String,
        value: String,
    ) -> Result<OrganizationAttribute, CoreError> {
        let upserted = self
            .inner
            .upsert_attribute(organization_id, key, value)
            .await;
        self.state.changed();
        upserted
    }

    async fn delete_attribute(
        &self,
        organization_id: OrganizationId,
        key: &str,
    ) -> Result<(), CoreError> {
        let deleted = self.inner.delete_attribute(organization_id, key).await;
        self.state.changed();
        deleted
    }
}

#[derive(Debug, Clone)]
pub struct CachedOrganizationMemberRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedOrganizationMemberRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: OrganizationMemberRepository> OrganizationMemberRepository
    for CachedOrganizationMemberRepository<R>
{
    async fn add_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<OrganizationMember, CoreError> {
        let added = self.inner.add_member(organization_id, user_id).await;
        self.state.changed();
        added
    }

    async fn remove_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let removed = self.inner.remove_member(organization_id, user_id).await;
        self.state.changed();
        removed
    }

    async fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<OrganizationMemberFilter, OrganizationMemberSortField>,
    ) -> Result<Page<OrganizationMember>, CoreError> {
        self.inner.list(organization, request).await
    }

    async fn list_organizations_for_user(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().organization_members,
                &(Uuid::from(realm_id), user_id),
                |_| true,
                || self.inner.list_organizations_for_user(realm_id, user_id),
            )
            .await
    }

    async fn get_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<Option<OrganizationMember>, CoreError> {
        self.inner.get_member(organization_id, user_id).await
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingOrganizationMemberRoleRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingOrganizationMemberRoleRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: OrganizationMemberRoleRepository> OrganizationMemberRoleRepository
    for InvalidatingOrganizationMemberRoleRepository<R>
{
    async fn assign_role(
        &self,
        organization_member_id: Uuid,
        role: &Scoped<Role>,
    ) -> Result<(), CoreError> {
        let assigned = self.inner.assign_role(organization_member_id, role).await;
        self.state.changed();
        assigned
    }

    async fn revoke_role(
        &self,
        organization_member_id: Uuid,
        role: &Scoped<Role>,
    ) -> Result<(), CoreError> {
        let revoked = self.inner.revoke_role(organization_member_id, role).await;
        self.state.changed();
        revoked
    }

    async fn list_role_ids(&self, organization_member_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        self.inner.list_role_ids(organization_member_id).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_organization::{
        MockOrganizationAttributeRepository, MockOrganizationMemberRepository,
        MockOrganizationMemberRoleRepository, MockOrganizationRepository,
    };

    use super::*;
    use crate::infrastructure::token_state::test_support::{
        role, scoped, synced_state, unsynced_state,
    };

    fn organization() -> Organization {
        let now = Utc::now();
        Organization {
            id: OrganizationId::new(Uuid::new_v4()),
            realm_id: RealmId::default(),
            name: "Acme".to_string(),
            alias: "acme".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn rename() -> UpdateOrganizationParams {
        UpdateOrganizationParams {
            name: Some("Acme 2".to_string()),
            alias: None,
            domain: None,
            redirect_url: None,
            description: None,
            enabled: None,
        }
    }

    fn finding_organization(stored: &Organization, calls: usize) -> MockOrganizationRepository {
        let stored = stored.clone();
        let mut inner = MockOrganizationRepository::new();
        inner
            .expect_get_organization_by_id()
            .times(calls)
            .returning(move |_| {
                let organization = Unscoped::new(stored.clone());
                Box::pin(async move { Ok(Some(organization)) })
            });
        inner
    }

    async fn read_organization(repository: &impl OrganizationRepository, id: OrganizationId) {
        repository
            .get_organization_by_id(id)
            .await
            .expect("read")
            .expect("present")
            .across_realms();
    }

    #[tokio::test]
    async fn an_organization_read_twice_is_loaded_once() {
        let stored = organization();
        let repository = CachedOrganizationRepository::new(
            Arc::new(finding_organization(&stored, 1)),
            synced_state(),
        );

        read_organization(&repository, stored.id).await;
        read_organization(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn updating_or_deleting_an_organization_makes_the_next_read_go_to_the_store() {
        let stored = organization();
        let mut inner = finding_organization(&stored, 3);
        let updated = stored.clone();
        inner
            .expect_update_organization()
            .times(1)
            .returning(move |_, _| {
                let organization = updated.clone();
                Box::pin(async move { Ok(organization) })
            });
        inner
            .expect_delete_organization()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = CachedOrganizationRepository::new(Arc::new(inner), synced_state());
        let target = scoped(stored.clone());
        read_organization(&repository, stored.id).await;

        repository
            .update_organization(&target, rename())
            .await
            .expect("update");
        read_organization(&repository, stored.id).await;
        repository
            .delete_organization(&target)
            .await
            .expect("delete");
        read_organization(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn a_missing_organization_is_asked_about_every_time() {
        let mut inner = MockOrganizationRepository::new();
        inner
            .expect_get_organization_by_id()
            .times(2)
            .returning(|_| Box::pin(async { Ok(None) }));
        let repository = CachedOrganizationRepository::new(Arc::new(inner), synced_state());
        let id = OrganizationId::new(Uuid::new_v4());

        assert!(
            repository
                .get_organization_by_id(id)
                .await
                .expect("first")
                .is_none()
        );
        assert!(
            repository
                .get_organization_by_id(id)
                .await
                .expect("second")
                .is_none()
        );
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_organization_read_goes_to_the_store() {
        let stored = organization();
        let repository = CachedOrganizationRepository::new(
            Arc::new(finding_organization(&stored, 2)),
            unsynced_state(),
        );

        read_organization(&repository, stored.id).await;
        read_organization(&repository, stored.id).await;
    }

    fn listing_attributes(calls: usize) -> MockOrganizationAttributeRepository {
        let mut inner = MockOrganizationAttributeRepository::new();
        inner
            .expect_list_attributes()
            .times(calls)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        inner
    }

    #[tokio::test]
    async fn writing_an_organization_attribute_makes_the_next_read_go_to_the_store() {
        let mut inner = listing_attributes(3);
        let id = OrganizationId::new(Uuid::new_v4());
        inner
            .expect_upsert_attribute()
            .times(1)
            .returning(move |_, key, value| {
                let attribute = OrganizationAttribute {
                    id: Uuid::new_v4(),
                    organization_id: id,
                    key,
                    value,
                    created_at: Utc::now(),
                };
                Box::pin(async move { Ok(attribute) })
            });
        inner
            .expect_delete_attribute()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository =
            CachedOrganizationAttributeRepository::new(Arc::new(inner), synced_state());
        repository.list_attributes(id).await.expect("warm");
        repository.list_attributes(id).await.expect("served");

        repository
            .upsert_attribute(id, "tier".to_string(), "gold".to_string())
            .await
            .expect("upsert");
        repository.list_attributes(id).await.expect("after upsert");
        repository
            .delete_attribute(id, "tier")
            .await
            .expect("delete");
        repository.list_attributes(id).await.expect("after delete");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_attribute_read_goes_to_the_store() {
        let repository = CachedOrganizationAttributeRepository::new(
            Arc::new(listing_attributes(2)),
            unsynced_state(),
        );
        let id = OrganizationId::new(Uuid::new_v4());

        repository.list_attributes(id).await.expect("first");
        repository.list_attributes(id).await.expect("second");
    }

    fn listing_memberships(calls: usize) -> MockOrganizationMemberRepository {
        let mut inner = MockOrganizationMemberRepository::new();
        inner
            .expect_list_organizations_for_user()
            .times(calls)
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
        inner
    }

    #[tokio::test]
    async fn membership_changes_make_the_next_read_go_to_the_store() {
        let mut inner = listing_memberships(3);
        let (organization_id, user_id) = (OrganizationId::new(Uuid::new_v4()), Uuid::new_v4());
        inner
            .expect_add_member()
            .times(1)
            .returning(|organization_id, user_id| {
                let member = OrganizationMember::new(organization_id, user_id);
                Box::pin(async move { Ok(member) })
            });
        inner
            .expect_remove_member()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository = CachedOrganizationMemberRepository::new(Arc::new(inner), synced_state());
        let realm_id = RealmId::default();
        repository
            .list_organizations_for_user(realm_id, user_id)
            .await
            .expect("warm");
        repository
            .list_organizations_for_user(realm_id, user_id)
            .await
            .expect("served");

        repository
            .add_member(organization_id, user_id)
            .await
            .expect("add");
        repository
            .list_organizations_for_user(realm_id, user_id)
            .await
            .expect("after add");
        repository
            .remove_member(organization_id, user_id)
            .await
            .expect("remove");
        repository
            .list_organizations_for_user(realm_id, user_id)
            .await
            .expect("after remove");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_membership_read_goes_to_the_store() {
        let repository = CachedOrganizationMemberRepository::new(
            Arc::new(listing_memberships(2)),
            unsynced_state(),
        );
        let (realm_id, user_id) = (RealmId::default(), Uuid::new_v4());

        for _ in 0..2 {
            repository
                .list_organizations_for_user(realm_id, user_id)
                .await
                .expect("read");
        }
    }

    #[tokio::test]
    async fn every_member_role_write_announces_a_change() {
        let state = synced_state();
        let mut inner = MockOrganizationMemberRoleRepository::new();
        inner
            .expect_assign_role()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        inner
            .expect_revoke_role()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository =
            InvalidatingOrganizationMemberRoleRepository::new(Arc::new(inner), state.clone());
        let target = scoped(role());

        repository
            .assign_role(Uuid::new_v4(), &target)
            .await
            .expect("assign");
        assert_eq!(state.generation(), 1);
        repository
            .revoke_role(Uuid::new_v4(), &target)
            .await
            .expect("revoke");
        assert_eq!(state.generation(), 2);
    }
}
