use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::user::entities::{
    RequiredAction, RequiredActionError, User, UserAttribute, UserFilter, UserSortField,
};
use ferriskey_domain::user::ports::{
    UserAttributeRepository, UserRepository, UserRequiredActionRepository, UserRoleRepository,
};
use ferriskey_domain::user::value_objects::{CreateUserRequest, UpdateUserRequest};
use uuid::Uuid;

use super::state::TokenState;

#[derive(Debug, Clone)]
pub struct CachedUserRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedUserRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: UserRepository> UserRepository for CachedUserRepository<R> {
    async fn create_user(&self, dto: CreateUserRequest) -> Result<User, CoreError> {
        self.inner.create_user(dto).await
    }

    async fn find_by_username(
        &self,
        username: String,
        realm_id: RealmId,
    ) -> Result<Option<User>, CoreError> {
        self.inner.find_by_username(username, realm_id).await
    }

    async fn get_by_username(
        &self,
        username: String,
        realm_id: RealmId,
    ) -> Result<User, CoreError> {
        self.inner.get_by_username(username, realm_id).await
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Unscoped<User>, CoreError> {
        self.inner.get_by_client_id(client_id).await
    }

    async fn get_by_id(&self, user_id: Uuid) -> Result<Unscoped<User>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().users,
                &user_id,
                |_| true,
                || self.inner.get_by_id(user_id),
            )
            .await
    }

    async fn get_by_id_without_relations(
        &self,
        user_id: Uuid,
    ) -> Result<Unscoped<User>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().bare_users,
                &user_id,
                |_| true,
                || self.inner.get_by_id_without_relations(user_id),
            )
            .await
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<UserFilter, UserSortField>,
    ) -> Result<Page<User>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn get_by_email(
        &self,
        email: &str,
        realm_id: RealmId,
    ) -> Result<Option<User>, CoreError> {
        self.inner.get_by_email(email, realm_id).await
    }

    async fn bulk_delete_user(&self, realm_id: RealmId, ids: Vec<Uuid>) -> Result<u64, CoreError> {
        let deleted = self.inner.bulk_delete_user(realm_id, ids).await;
        self.state.changed();
        deleted
    }

    async fn delete_user(&self, user: &Scoped<User>) -> Result<u64, CoreError> {
        let deleted = self.inner.delete_user(user).await;
        self.state.changed();
        deleted
    }

    async fn update_user(
        &self,
        user: &Scoped<User>,
        dto: UpdateUserRequest,
    ) -> Result<User, CoreError> {
        let updated = self.inner.update_user(user, dto).await;
        self.state.changed();
        updated
    }

    async fn update_locale(
        &self,
        user: &Scoped<User>,
        locale: Option<String>,
    ) -> Result<User, CoreError> {
        let updated = self.inner.update_locale(user, locale).await;
        self.state.changed();
        updated
    }

    async fn increment_failed_login_attempts(
        &self,
        user_id: Uuid,
        threshold: i32,
        locked_until: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        self.inner
            .increment_failed_login_attempts(user_id, threshold, locked_until)
            .await
    }

    async fn reset_failed_login_attempts(&self, user_id: Uuid) -> Result<(), CoreError> {
        self.inner.reset_failed_login_attempts(user_id).await
    }

    async fn unlock_user(&self, user: &Scoped<User>) -> Result<(), CoreError> {
        self.inner.unlock_user(user).await
    }
}

#[derive(Debug, Clone)]
pub struct CachedUserRoleRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedUserRoleRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: UserRoleRepository> UserRoleRepository for CachedUserRoleRepository<R> {
    async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        let assigned = self.inner.assign_role(user_id, role_id).await;
        self.state.changed();
        assigned
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        let revoked = self.inner.revoke_role(user_id, role_id).await;
        self.state.changed();
        revoked
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().user_roles,
                &user_id,
                |_| true,
                || self.inner.get_user_roles(user_id),
            )
            .await
    }

    async fn get_service_account_roles(&self, client_id: Uuid) -> Result<Vec<Role>, CoreError> {
        self.inner.get_service_account_roles(client_id).await
    }

    async fn get_roles_by_ids(&self, mut role_ids: Vec<Uuid>) -> Result<Vec<Role>, CoreError> {
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }

        role_ids.sort_unstable();
        role_ids.dedup();
        self.state
            .read_subject(
                &self.state.subject().roles_by_ids,
                &role_ids,
                |_| true,
                || self.inner.get_roles_by_ids(role_ids.clone()),
            )
            .await
    }

    async fn has_role(&self, user_id: Uuid, role_id: Uuid) -> Result<bool, CoreError> {
        self.inner.has_role(user_id, role_id).await
    }
}

#[derive(Debug, Clone)]
pub struct CachedUserAttributeRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedUserAttributeRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: UserAttributeRepository> UserAttributeRepository for CachedUserAttributeRepository<R> {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserAttribute>, CoreError> {
        self.state
            .read_subject(
                &self.state.subject().user_attributes,
                &user_id,
                |_| true,
                || self.inner.list_by_user_id(user_id),
            )
            .await
    }

    async fn upsert_many(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        attributes: HashMap<String, String>,
    ) -> Result<Vec<UserAttribute>, CoreError> {
        let upserted = self.inner.upsert_many(user_id, realm_id, attributes).await;
        self.state.changed();
        upserted
    }

    async fn delete_by_key(&self, user_id: Uuid, key: String) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_key(user_id, key).await;
        self.state.changed();
        deleted
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingUserRequiredActionRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingUserRequiredActionRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: UserRequiredActionRepository> UserRequiredActionRepository
    for InvalidatingUserRequiredActionRepository<R>
{
    async fn add_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> Result<(), RequiredActionError> {
        let added = self.inner.add_required_action(user_id, action).await;
        self.state.changed();
        added
    }

    async fn remove_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> Result<(), RequiredActionError> {
        let removed = self.inner.remove_required_action(user_id, action).await;
        self.state.changed();
        removed
    }

    async fn get_required_actions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RequiredAction>, RequiredActionError> {
        self.inner.get_required_actions(user_id).await
    }

    async fn clear_required_actions(&self, user_id: Uuid) -> Result<u64, RequiredActionError> {
        let cleared = self.inner.clear_required_actions(user_id).await;
        self.state.changed();
        cleared
    }
}

#[cfg(test)]
mod tests {
    use ferriskey_domain::user::ports::{
        MockUserAttributeRepository, MockUserRepository, MockUserRequiredActionRepository,
        MockUserRoleRepository,
    };

    use super::*;
    use crate::infrastructure::token_state::test_support::{
        disabled_state, role, scoped, synced_state, unsynced_state, user,
    };

    fn finding_user(stored: &User, calls: usize) -> MockUserRepository {
        let stored = stored.clone();
        let mut inner = MockUserRepository::new();
        let by_id = stored.clone();
        inner.expect_get_by_id().times(calls).returning(move |_| {
            let user = Unscoped::new(by_id.clone());
            Box::pin(async move { Ok(user) })
        });
        inner
            .expect_get_by_id_without_relations()
            .times(calls)
            .returning(move |_| {
                let user = Unscoped::new(stored.clone());
                Box::pin(async move { Ok(user) })
            });
        inner
    }

    async fn read_both(repository: &impl UserRepository, id: Uuid) {
        repository
            .get_by_id(id)
            .await
            .expect("get_by_id")
            .across_realms();
        repository
            .get_by_id_without_relations(id)
            .await
            .expect("get_by_id_without_relations")
            .across_realms();
    }

    #[tokio::test]
    async fn a_user_read_twice_is_loaded_once() {
        let stored = user();
        let repository =
            CachedUserRepository::new(Arc::new(finding_user(&stored, 1)), synced_state());

        read_both(&repository, stored.id).await;
        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn updating_a_user_makes_the_next_read_go_to_the_store() {
        let stored = user();
        let mut inner = finding_user(&stored, 2);
        let updated = stored.clone();
        inner.expect_update_user().times(1).returning(move |_, _| {
            let updated = updated.clone();
            Box::pin(async move { Ok(updated) })
        });
        let repository = CachedUserRepository::new(Arc::new(inner), synced_state());
        read_both(&repository, stored.id).await;

        repository
            .update_user(
                &scoped(stored.clone()),
                UpdateUserRequest {
                    username: None,
                    firstname: None,
                    lastname: None,
                    email: None,
                    email_verified: true,
                    enabled: false,
                    required_actions: None,
                },
            )
            .await
            .expect("update");

        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn changing_the_locale_makes_the_next_read_go_to_the_store() {
        let stored = user();
        let mut inner = finding_user(&stored, 2);
        let updated = stored.clone();
        inner
            .expect_update_locale()
            .times(1)
            .returning(move |_, _| {
                let updated = updated.clone();
                Box::pin(async move { Ok(updated) })
            });
        let repository = CachedUserRepository::new(Arc::new(inner), synced_state());
        read_both(&repository, stored.id).await;

        repository
            .update_locale(&scoped(stored.clone()), Some("fr".to_string()))
            .await
            .expect("update locale");

        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn deleting_a_user_makes_the_next_read_go_to_the_store() {
        let stored = user();
        let mut inner = finding_user(&stored, 2);
        inner
            .expect_delete_user()
            .times(1)
            .returning(|_| Box::pin(async { Ok(1) }));
        inner
            .expect_bulk_delete_user()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(1) }));
        let repository = CachedUserRepository::new(Arc::new(inner), synced_state());
        read_both(&repository, stored.id).await;

        repository
            .delete_user(&scoped(stored.clone()))
            .await
            .expect("delete");
        read_both(&repository, stored.id).await;
        repository
            .bulk_delete_user(stored.realm_id, vec![stored.id])
            .await
            .expect("bulk delete");
    }

    #[tokio::test]
    async fn lockout_writes_keep_the_cache() {
        let stored = user();
        let mut inner = finding_user(&stored, 1);
        inner
            .expect_increment_failed_login_attempts()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));
        inner
            .expect_reset_failed_login_attempts()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        inner
            .expect_unlock_user()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = CachedUserRepository::new(Arc::new(inner), synced_state());
        read_both(&repository, stored.id).await;

        repository
            .increment_failed_login_attempts(stored.id, 3, Utc::now())
            .await
            .expect("increment");
        repository
            .reset_failed_login_attempts(stored.id)
            .await
            .expect("reset");
        repository
            .unlock_user(&scoped(stored.clone()))
            .await
            .expect("unlock");

        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_user_read_goes_to_the_store() {
        let stored = user();
        let repository =
            CachedUserRepository::new(Arc::new(finding_user(&stored, 2)), unsynced_state());

        read_both(&repository, stored.id).await;
        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn without_a_ttl_every_user_read_goes_to_the_store() {
        let stored = user();
        let repository =
            CachedUserRepository::new(Arc::new(finding_user(&stored, 2)), disabled_state());

        read_both(&repository, stored.id).await;
        read_both(&repository, stored.id).await;
    }

    #[tokio::test]
    async fn a_user_read_overtaken_by_a_change_is_not_kept() {
        let stored = user();
        let state = synced_state();
        let racing = Arc::clone(&state);
        let mut inner = MockUserRepository::new();
        inner.expect_get_by_id().times(2).returning(move |_| {
            racing.changed();
            let user = Unscoped::new(stored.clone());
            Box::pin(async move { Ok(user) })
        });
        let repository = CachedUserRepository::new(Arc::new(inner), state);

        for _ in 0..2 {
            repository
                .get_by_id(Uuid::nil())
                .await
                .expect("read")
                .across_realms();
        }
    }

    fn finding_roles(calls: usize) -> MockUserRoleRepository {
        let mut inner = MockUserRoleRepository::new();
        inner.expect_get_user_roles().times(calls).returning(|_| {
            let roles = vec![role()];
            Box::pin(async move { Ok(roles) })
        });
        inner.expect_get_roles_by_ids().times(calls).returning(|_| {
            let roles = vec![role()];
            Box::pin(async move { Ok(roles) })
        });
        inner
    }

    async fn read_roles(repository: &impl UserRoleRepository, user_id: Uuid, ids: Vec<Uuid>) {
        repository.get_user_roles(user_id).await.expect("roles");
        repository.get_roles_by_ids(ids).await.expect("by ids");
    }

    #[tokio::test]
    async fn roles_are_loaded_once_whatever_the_order_of_the_requested_ids() {
        let repository = CachedUserRoleRepository::new(Arc::new(finding_roles(1)), synced_state());
        let (a, b, user_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

        read_roles(&repository, user_id, vec![a, b]).await;
        read_roles(&repository, user_id, vec![b, a, b]).await;
    }

    #[tokio::test]
    async fn assigning_or_revoking_a_role_makes_the_next_read_go_to_the_store() {
        let mut inner = finding_roles(3);
        inner
            .expect_assign_role()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        inner
            .expect_revoke_role()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository = CachedUserRoleRepository::new(Arc::new(inner), synced_state());
        let (user_id, role_id) = (Uuid::new_v4(), Uuid::new_v4());
        read_roles(&repository, user_id, vec![role_id]).await;

        repository
            .assign_role(user_id, role_id)
            .await
            .expect("assign");
        read_roles(&repository, user_id, vec![role_id]).await;
        repository
            .revoke_role(user_id, role_id)
            .await
            .expect("revoke");
        read_roles(&repository, user_id, vec![role_id]).await;
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_role_read_goes_to_the_store() {
        let repository =
            CachedUserRoleRepository::new(Arc::new(finding_roles(2)), unsynced_state());
        let user_id = Uuid::new_v4();

        read_roles(&repository, user_id, vec![Uuid::new_v4()]).await;
        read_roles(&repository, user_id, vec![Uuid::new_v4()]).await;
    }

    #[tokio::test]
    async fn an_empty_role_id_list_is_answered_without_the_store() {
        let repository =
            CachedUserRoleRepository::new(Arc::new(MockUserRoleRepository::new()), synced_state());

        assert!(
            repository
                .get_roles_by_ids(Vec::new())
                .await
                .expect("by ids")
                .is_empty()
        );
    }

    fn listing_attributes(calls: usize) -> MockUserAttributeRepository {
        let mut inner = MockUserAttributeRepository::new();
        inner
            .expect_list_by_user_id()
            .times(calls)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        inner
    }

    #[tokio::test]
    async fn writing_user_attributes_makes_the_next_read_go_to_the_store() {
        let mut inner = listing_attributes(3);
        inner
            .expect_upsert_many()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));
        inner
            .expect_delete_by_key()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let repository = CachedUserAttributeRepository::new(Arc::new(inner), synced_state());
        let user_id = Uuid::new_v4();
        repository.list_by_user_id(user_id).await.expect("warm");
        repository.list_by_user_id(user_id).await.expect("served");

        repository
            .upsert_many(user_id, RealmId::default(), HashMap::new())
            .await
            .expect("upsert");
        repository
            .list_by_user_id(user_id)
            .await
            .expect("after upsert");
        repository
            .delete_by_key(user_id, "k".to_string())
            .await
            .expect("delete");
        repository
            .list_by_user_id(user_id)
            .await
            .expect("after delete");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_attribute_read_goes_to_the_store() {
        let repository =
            CachedUserAttributeRepository::new(Arc::new(listing_attributes(2)), unsynced_state());
        let user_id = Uuid::new_v4();

        repository.list_by_user_id(user_id).await.expect("first");
        repository.list_by_user_id(user_id).await.expect("second");
    }

    #[tokio::test]
    async fn required_action_writes_make_the_next_user_read_go_to_the_store() {
        let state = synced_state();
        let stored = user();
        let users = CachedUserRepository::new(Arc::new(finding_user(&stored, 2)), state.clone());
        let mut inner = MockUserRequiredActionRepository::new();
        inner
            .expect_add_required_action()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        inner
            .expect_remove_required_action()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        inner
            .expect_clear_required_actions()
            .times(1)
            .returning(|_| Box::pin(async { Ok(0) }));
        let actions = InvalidatingUserRequiredActionRepository::new(Arc::new(inner), state);
        read_both(&users, stored.id).await;

        actions
            .add_required_action(stored.id, RequiredAction::UpdatePassword)
            .await
            .expect("add");
        read_both(&users, stored.id).await;
        actions
            .remove_required_action(stored.id, RequiredAction::UpdatePassword)
            .await
            .expect("remove");
        actions
            .clear_required_actions(stored.id)
            .await
            .expect("clear");
    }
}
