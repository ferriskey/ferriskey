use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_organization::ports::TokenGroupMembership;
use uuid::Uuid;

use crate::domain::role::entities::Role;
use crate::domain::user::entities::User;
use crate::infrastructure::common::{Retention, TtlCache};

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

    pub async fn load<E, F, Fut>(&self, client_id: &Uuid, load: F) -> Result<Arc<ServiceAccount>, E>
    where
        E: Clone + Send + Sync + 'static,
        F: Fn() -> Fut,
        Fut: Future<Output = Result<(Arc<ServiceAccount>, Retention), E>>,
    {
        self.entries.load(client_id, load).await
    }
}
