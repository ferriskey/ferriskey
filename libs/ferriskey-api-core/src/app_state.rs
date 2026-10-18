use std::sync::Arc;

use ferriskey_core::ApplicationService;
use ferriskey_core::domain::common::rate_limit::{
    CLIENT_AUTH_FAILURES_PER_MINUTE, ClientAuthThrottle,
};

use crate::args::Args;
use crate::cors::{RealmOriginCache, WEB_ORIGIN_CACHE_TTL};

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: ApplicationService,
    pub web_origin_cache: Arc<RealmOriginCache>,
    pub client_auth_throttle: Arc<ClientAuthThrottle>,
}

impl AppState {
    pub fn new(args: Arc<Args>, service: ApplicationService) -> Self {
        Self {
            args,
            service,
            web_origin_cache: Arc::new(RealmOriginCache::new(WEB_ORIGIN_CACHE_TTL)),
            client_auth_throttle: Arc::new(ClientAuthThrottle::per_minute(
                CLIENT_AUTH_FAILURES_PER_MINUTE,
            )),
        }
    }
}
