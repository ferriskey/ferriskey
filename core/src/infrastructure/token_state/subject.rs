use std::future::Future;
use std::time::Duration;

use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::user::entities::{User, UserAttribute};
use ferriskey_organization::{
    Organization, OrganizationAttribute, OrganizationMember, TokenGroupMembership,
};
use uuid::Uuid;

use super::state::TokenState;
use crate::infrastructure::common::{Retention, TtlCache};

#[derive(Debug)]
pub(super) struct SubjectCaches {
    pub(super) users: TtlCache<Uuid, Unscoped<User>>,
    pub(super) bare_users: TtlCache<Uuid, Unscoped<User>>,
    pub(super) user_roles: TtlCache<Uuid, Vec<Role>>,
    pub(super) roles_by_ids: TtlCache<Vec<Uuid>, Vec<Role>>,
    pub(super) memberships: TtlCache<Uuid, TokenGroupMembership>,
    pub(super) organization_members: TtlCache<(Uuid, Uuid), Vec<OrganizationMember>>,
    pub(super) organizations: TtlCache<Uuid, Option<Unscoped<Organization>>>,
    pub(super) organization_attributes: TtlCache<Uuid, Vec<OrganizationAttribute>>,
    pub(super) user_attributes: TtlCache<Uuid, Vec<UserAttribute>>,
}

impl SubjectCaches {
    pub(super) fn new(ttl: Duration) -> Self {
        Self {
            users: TtlCache::new(ttl),
            bare_users: TtlCache::new(ttl),
            user_roles: TtlCache::new(ttl),
            roles_by_ids: TtlCache::new(ttl),
            memberships: TtlCache::new(ttl),
            organization_members: TtlCache::new(ttl),
            organizations: TtlCache::new(ttl),
            organization_attributes: TtlCache::new(ttl),
            user_attributes: TtlCache::new(ttl),
        }
    }

    pub(super) fn clear(&self) {
        self.users.clear();
        self.bare_users.clear();
        self.user_roles.clear();
        self.roles_by_ids.clear();
        self.memberships.clear();
        self.organization_members.clear();
        self.organizations.clear();
        self.organization_attributes.clear();
        self.user_attributes.clear();
    }
}

impl TokenState {
    pub(super) async fn read_subject<K, V, E, F, Fut>(
        &self,
        cache: &TtlCache<K, V>,
        key: &K,
        keepable: fn(&V) -> bool,
        load: F,
    ) -> Result<V, E>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        E: Clone + Send + Sync + 'static,
        F: Fn() -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        if !self.synced() {
            return load().await;
        }

        let read_at = self.generation();
        let load = &load;
        cache
            .load_while(
                key,
                move || self.generation() == read_at,
                move || async move {
                    let value = load().await?;
                    let retention = if keepable(&value) && self.generation() == read_at {
                        Retention::Keep
                    } else {
                        Retention::Own
                    };

                    Ok((value, retention))
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use uuid::Uuid;

    use crate::infrastructure::token_state::TokenState;
    use crate::infrastructure::token_state::test_support::{
        disabled_state, synced_state, unsynced_state,
    };

    async fn read_twice(state: &Arc<TokenState>) -> usize {
        let loads = AtomicUsize::new(0);
        let id = Uuid::nil();
        for _ in 0..2 {
            state
                .read_subject(
                    &state.subject().user_roles,
                    &id,
                    |_| true,
                    || async {
                        loads.fetch_add(1, Ordering::SeqCst);
                        Ok::<_, String>(Vec::new())
                    },
                )
                .await
                .expect("read");
        }
        loads.load(Ordering::SeqCst)
    }

    #[tokio::test]
    async fn a_synced_state_loads_once() {
        assert_eq!(read_twice(&synced_state()).await, 1);
    }

    #[tokio::test]
    async fn spec_rtp_18_while_the_state_is_not_in_sync_every_subject_read_hits_the_store() {
        assert_eq!(read_twice(&unsynced_state()).await, 2);
    }

    #[tokio::test]
    async fn spec_rtp_18_with_a_zero_ttl_every_subject_read_hits_the_store() {
        assert_eq!(read_twice(&disabled_state()).await, 2);
    }

    #[tokio::test]
    async fn a_value_the_caller_deems_unkeepable_is_loaded_every_time() {
        let state = synced_state();
        let loads = AtomicUsize::new(0);
        let id = Uuid::nil();
        for _ in 0..2 {
            state
                .read_subject(
                    &state.subject().organizations,
                    &id,
                    Option::is_some,
                    || async {
                        loads.fetch_add(1, Ordering::SeqCst);
                        Ok::<_, String>(None)
                    },
                )
                .await
                .expect("read");
        }

        assert_eq!(loads.load(Ordering::SeqCst), 2);
    }
}
