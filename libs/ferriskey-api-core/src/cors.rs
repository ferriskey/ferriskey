use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use ferriskey_core::domain::client::entities::web_origin::{InvalidOrigin, Origin};
use ferriskey_core::domain::client::ports::WebOriginResolver;
use tracing::warn;

pub const WEB_ORIGIN_CACHE_TTL: Duration = Duration::from_secs(30);

type CachedOrigins = (Instant, Arc<HashSet<Origin>>);

pub struct RealmOriginCache {
    entries: RwLock<HashMap<String, CachedOrigins>>,
    ttl: Duration,
}

impl RealmOriginCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub fn get_fresh(&self, realm_name: &str, now: Instant) -> Option<Arc<HashSet<Origin>>> {
        let entries = self.entries.read().ok()?;
        let (stored_at, origins) = entries.get(realm_name)?;

        (now.duration_since(*stored_at) < self.ttl).then(|| Arc::clone(origins))
    }

    pub fn store(&self, realm_name: String, origins: Arc<HashSet<Origin>>, now: Instant) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(realm_name, (now, origins));
        }
    }

    pub fn invalidate(&self, realm_name: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(realm_name);
        }
    }
}

pub fn origin_is_allowed(
    static_origins: &HashSet<Origin>,
    realm_origins: Option<&HashSet<Origin>>,
    raw_origin: &str,
) -> bool {
    let Ok(origin) = Origin::try_from(raw_origin) else {
        return false;
    };

    if static_origins.contains(&origin) {
        return true;
    }

    realm_origins.is_some_and(|origins| origins.contains(&origin))
}

pub fn realm_from_path<'a>(root_path: &str, path: &'a str) -> Option<&'a str> {
    let remainder = path.strip_prefix(root_path)?;
    let remainder = remainder.strip_prefix("/realms/")?;

    let realm = remainder.split('/').next()?;

    (!realm.is_empty()).then_some(realm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_realm_from_a_protocol_path() {
        assert_eq!(
            realm_from_path("", "/realms/master/protocol/openid-connect/token"),
            Some("master")
        );
    }

    #[test]
    fn extracts_the_realm_from_a_management_path() {
        assert_eq!(
            realm_from_path("", "/realms/tenant-a/clients"),
            Some("tenant-a")
        );
    }

    #[test]
    fn extracts_the_realm_under_a_root_path() {
        assert_eq!(
            realm_from_path("/auth", "/auth/realms/master/users"),
            Some("master")
        );
    }

    #[test]
    fn extracts_the_realm_when_it_is_the_last_segment() {
        assert_eq!(realm_from_path("", "/realms/master"), Some("master"));
    }

    #[test]
    fn tolerates_a_trailing_slash_after_the_realm() {
        assert_eq!(realm_from_path("", "/realms/master/"), Some("master"));
    }

    #[test]
    fn finds_no_realm_on_a_path_that_has_none() {
        assert_eq!(realm_from_path("", "/config"), None);
        assert_eq!(realm_from_path("", "/metrics"), None);
    }

    #[test]
    fn finds_no_realm_on_the_realms_collection() {
        assert_eq!(realm_from_path("", "/realms"), None);
    }

    #[test]
    fn refuses_an_empty_realm_name() {
        assert_eq!(realm_from_path("", "/realms/"), None);
        assert_eq!(realm_from_path("", "/realms//clients"), None);
    }

    #[test]
    fn refuses_a_path_that_does_not_start_with_the_root_path() {
        assert_eq!(realm_from_path("/auth", "/realms/master/users"), None);
    }

    #[test]
    fn refuses_a_realms_segment_that_is_not_where_the_router_puts_it() {
        assert_eq!(realm_from_path("", "/foo/realms/master/users"), None);
    }

    #[test]
    fn does_not_decode_a_percent_encoded_realm_name() {
        assert_eq!(
            realm_from_path("", "/realms/my%2Frealm/users"),
            Some("my%2Frealm")
        );
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::try_from(value).expect("valid test origin")
    }

    fn origins(values: &[&str]) -> Arc<HashSet<Origin>> {
        Arc::new(values.iter().map(|value| origin(value)).collect())
    }

    #[test]
    fn holds_nothing_before_anything_is_stored() {
        let cache = RealmOriginCache::new(Duration::from_secs(30));

        assert!(cache.get_fresh("master", Instant::now()).is_none());
    }

    #[test]
    fn serves_what_was_stored_while_the_ttl_holds() {
        let cache = RealmOriginCache::new(Duration::from_secs(30));
        let stored_at = Instant::now();

        cache.store(
            "master".to_string(),
            origins(&["https://app.example.com"]),
            stored_at,
        );

        let served = cache
            .get_fresh("master", stored_at + Duration::from_secs(29))
            .expect("entry is still fresh");
        assert!(served.contains(&origin("https://app.example.com")));
    }

    #[test]
    fn stops_serving_once_the_ttl_has_elapsed() {
        let cache = RealmOriginCache::new(Duration::from_secs(30));
        let stored_at = Instant::now();

        cache.store(
            "master".to_string(),
            origins(&["https://app.example.com"]),
            stored_at,
        );

        assert!(
            cache
                .get_fresh("master", stored_at + Duration::from_secs(31))
                .is_none()
        );
    }

    #[test]
    fn invalidating_forgets_the_realm_immediately() {
        let cache = RealmOriginCache::new(Duration::from_secs(30));
        let now = Instant::now();

        cache.store(
            "master".to_string(),
            origins(&["https://app.example.com"]),
            now,
        );
        cache.invalidate("master");

        assert!(cache.get_fresh("master", now).is_none());
    }

    #[test]
    fn one_realm_never_serves_another_realms_entry() {
        let cache = RealmOriginCache::new(Duration::from_secs(30));
        let now = Instant::now();

        cache.store(
            "tenant-a".to_string(),
            origins(&["https://a.example.com"]),
            now,
        );

        assert!(cache.get_fresh("tenant-b", now).is_none());
    }
}

#[cfg(test)]
mod decision_tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::try_from(value).expect("valid test origin")
    }

    fn set(values: &[&str]) -> HashSet<Origin> {
        values.iter().map(|value| origin(value)).collect()
    }

    #[test]
    fn an_origin_from_the_environment_is_allowed_everywhere() {
        let static_origins = set(&["https://console.example.com"]);

        assert!(origin_is_allowed(
            &static_origins,
            None,
            "https://console.example.com"
        ));
    }

    #[test]
    fn an_origin_registered_by_the_realm_is_allowed() {
        let realm_origins = set(&["https://app.example.com"]);

        assert!(origin_is_allowed(
            &HashSet::new(),
            Some(&realm_origins),
            "https://app.example.com"
        ));
    }

    #[test]
    fn an_origin_registered_by_a_different_realm_is_refused() {
        let realm_origins = set(&["https://b.example.com"]);

        assert!(!origin_is_allowed(
            &HashSet::new(),
            Some(&realm_origins),
            "https://a.example.com"
        ));
    }

    #[test]
    fn an_origin_nobody_registered_is_refused() {
        assert!(!origin_is_allowed(
            &HashSet::new(),
            Some(&set(&["https://app.example.com"])),
            "https://attacker.example.com"
        ));
    }

    #[test]
    fn a_realm_origin_does_not_apply_where_the_path_carries_no_realm() {
        assert!(!origin_is_allowed(
            &HashSet::new(),
            None,
            "https://app.example.com"
        ));
    }

    #[test]
    fn a_header_that_is_not_an_origin_is_refused() {
        assert!(!origin_is_allowed(&HashSet::new(), None, "not an origin"));
        assert!(!origin_is_allowed(
            &set(&["https://app.example.com"]),
            None,
            "null"
        ));
    }

    #[test]
    fn the_wildcard_is_refused_even_if_something_stored_it() {
        assert!(!origin_is_allowed(&HashSet::new(), None, "*"));
    }

    #[test]
    fn an_origin_is_matched_after_normalization() {
        let static_origins = set(&["https://app.example.com"]);

        assert!(origin_is_allowed(
            &static_origins,
            None,
            "https://app.example.com:443"
        ));
    }
}

pub struct CorsOriginResolver<R> {
    resolver: Arc<R>,
    root_path: String,
    static_origins: HashSet<Origin>,
    cache: Arc<RealmOriginCache>,
}

impl<R: WebOriginResolver> CorsOriginResolver<R> {
    pub fn new(
        resolver: Arc<R>,
        root_path: String,
        configured_origins: &[String],
        cache: Arc<RealmOriginCache>,
    ) -> Result<Self, InvalidOrigin> {
        let static_origins = configured_origins
            .iter()
            .map(|value| Origin::try_from(value.as_str()))
            .collect::<Result<HashSet<Origin>, InvalidOrigin>>()?;

        Ok(Self {
            resolver,
            root_path,
            static_origins,
            cache,
        })
    }

    pub fn invalidate(&self, realm_name: &str) {
        self.cache.invalidate(realm_name);
    }

    pub async fn is_allowed(&self, path: &str, raw_origin: &str) -> bool {
        let Ok(origin) = Origin::try_from(raw_origin) else {
            return false;
        };

        if self.static_origins.contains(&origin) {
            return true;
        }

        let Some(realm_name) = realm_from_path(&self.root_path, path) else {
            return false;
        };

        self.realm_origins(realm_name)
            .await
            .is_some_and(|origins| origins.contains(&origin))
    }

    async fn realm_origins(&self, realm_name: &str) -> Option<Arc<HashSet<Origin>>> {
        let now = Instant::now();

        if let Some(cached) = self.cache.get_fresh(realm_name, now) {
            return Some(cached);
        }

        match self
            .resolver
            .resolve_realm_origins(realm_name.to_string())
            .await
        {
            Ok(origins) => {
                let origins = Arc::new(origins);
                self.cache
                    .store(realm_name.to_string(), Arc::clone(&origins), now);
                Some(origins)
            }
            Err(error) => {
                warn!(realm = realm_name, %error, "could not resolve web origins, refusing cross-origin request");
                None
            }
        }
    }
}

#[cfg(test)]
mod resolver_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use ferriskey_core::domain::common::entities::app_errors::CoreError;

    use super::*;

    struct FakeStore {
        origins: HashMap<String, Vec<String>>,
        calls: AtomicUsize,
        fails: bool,
    }

    impl FakeStore {
        fn holding(realm: &str, origins: &[&str]) -> Self {
            Self {
                origins: HashMap::from([(
                    realm.to_string(),
                    origins.iter().map(|value| value.to_string()).collect(),
                )]),
                calls: AtomicUsize::new(0),
                fails: false,
            }
        }

        fn failing() -> Self {
            Self {
                origins: HashMap::new(),
                calls: AtomicUsize::new(0),
                fails: true,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl WebOriginResolver for FakeStore {
        async fn resolve_realm_origins(
            &self,
            realm_name: String,
        ) -> Result<HashSet<Origin>, CoreError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            if self.fails {
                return Err(CoreError::InternalServerError);
            }

            Ok(self
                .origins
                .get(&realm_name)
                .map(|values| {
                    values
                        .iter()
                        .map(|value| Origin::try_from(value.as_str()).expect("valid test origin"))
                        .collect()
                })
                .unwrap_or_default())
        }
    }

    fn resolver(store: Arc<FakeStore>, configured: &[String]) -> CorsOriginResolver<FakeStore> {
        CorsOriginResolver::new(
            store,
            String::new(),
            configured,
            Arc::new(RealmOriginCache::new(Duration::from_secs(30))),
        )
        .expect("configured origins are valid")
    }

    #[tokio::test]
    async fn allows_an_origin_the_realm_registered() {
        let store = Arc::new(FakeStore::holding("tenant-a", &["https://a.example.com"]));
        let resolver = resolver(store, &[]);

        assert!(
            resolver
                .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
                .await
        );
    }

    #[tokio::test]
    async fn refuses_that_same_origin_on_another_realms_path() {
        let store = Arc::new(FakeStore::holding("tenant-a", &["https://a.example.com"]));
        let resolver = resolver(store, &[]);

        assert!(
            !resolver
                .is_allowed("/realms/tenant-b/clients", "https://a.example.com")
                .await
        );
    }

    #[tokio::test]
    async fn allows_a_configured_origin_where_the_path_carries_no_realm() {
        let store = Arc::new(FakeStore::holding("tenant-a", &[]));
        let resolver = resolver(store.clone(), &["https://console.example.com".to_string()]);

        assert!(
            resolver
                .is_allowed("/config", "https://console.example.com")
                .await
        );
        assert_eq!(store.calls(), 0);
    }

    #[tokio::test]
    async fn refuses_everything_when_the_store_fails() {
        let store = Arc::new(FakeStore::failing());
        let resolver = resolver(store, &[]);

        assert!(
            !resolver
                .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
                .await
        );
    }

    #[tokio::test]
    async fn reaches_the_store_once_per_realm_while_the_entry_is_fresh() {
        let store = Arc::new(FakeStore::holding("tenant-a", &["https://a.example.com"]));
        let resolver = resolver(store.clone(), &[]);

        for _ in 0..5 {
            resolver
                .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
                .await;
        }

        assert_eq!(store.calls(), 1);
    }

    #[tokio::test]
    async fn does_not_cache_a_failure() {
        let store = Arc::new(FakeStore::failing());
        let resolver = resolver(store.clone(), &[]);

        for _ in 0..3 {
            resolver
                .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
                .await;
        }

        assert_eq!(store.calls(), 3);
    }

    #[tokio::test]
    async fn invalidating_makes_the_next_request_read_the_store_again() {
        let store = Arc::new(FakeStore::holding("tenant-a", &["https://a.example.com"]));
        let resolver = resolver(store.clone(), &[]);

        resolver
            .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
            .await;
        resolver.invalidate("tenant-a");
        resolver
            .is_allowed("/realms/tenant-a/clients", "https://a.example.com")
            .await;

        assert_eq!(store.calls(), 2);
    }

    #[test]
    fn a_configured_origin_that_is_not_an_origin_is_refused_at_startup() {
        let store = Arc::new(FakeStore::holding("tenant-a", &[]));

        let outcome = CorsOriginResolver::new(
            store,
            String::new(),
            &["https://console.example.com/app".to_string()],
            Arc::new(RealmOriginCache::new(Duration::from_secs(30))),
        )
        .map(|_| ());

        assert_eq!(outcome, Err(InvalidOrigin::NotAnOrigin));
    }
}
