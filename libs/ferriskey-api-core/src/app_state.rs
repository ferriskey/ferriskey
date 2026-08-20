use std::sync::Arc;

use ferriskey_core::ApplicationService;

use crate::args::Args;
use crate::cors::{RealmOriginCache, WEB_ORIGIN_CACHE_TTL};

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: ApplicationService,
    pub web_origin_cache: Arc<RealmOriginCache>,
}

impl AppState {
    pub fn new(args: Arc<Args>, service: ApplicationService) -> Self {
        Self {
            args,
            service,
            web_origin_cache: Arc::new(RealmOriginCache::new(WEB_ORIGIN_CACHE_TTL)),
        }
    }
}
