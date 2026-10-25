use std::sync::Arc;
use std::time::Duration;

use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_organization::ports::TokenGroupMembership;
use uuid::Uuid;

use crate::domain::role::entities::Role;
use crate::domain::user::entities::User;
use crate::infrastructure::common::TtlCache;

#[derive(Debug)]
pub struct ServiceAccount {
    pub user: Unscoped<User>,
    pub roles: Vec<Role>,
    pub membership: TokenGroupMembership,
}

#[derive(Debug)]
pub struct ServiceAccountCache {
    entries: TtlCache<Uuid, Arc<ServiceAccount>>,
}

impl ServiceAccountCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: TtlCache::new(ttl),
        }
    }

    pub fn get(&self, client_id: &Uuid) -> Option<Arc<ServiceAccount>> {
        self.entries.get(client_id)
    }

    pub fn insert(&self, client_id: Uuid, account: Arc<ServiceAccount>) {
        self.entries.insert(client_id, account);
    }
}
