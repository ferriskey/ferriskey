use std::sync::Arc;

use crate::infrastructure::aegis::cache::AegisConfigCache;
use crate::infrastructure::client::cache::ClientCaches;
use crate::infrastructure::realm::cache::RealmCaches;

#[derive(Debug, Clone)]
pub struct ConfigCaches {
    pub realm: RealmCaches,
    pub client: ClientCaches,
    pub aegis: Arc<AegisConfigCache>,
}

impl ConfigCaches {
    pub fn clear(&self) {
        self.realm.clear();
        self.client.clear();
        self.aegis.clear();
    }
}
