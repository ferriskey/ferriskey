use std::borrow::Borrow;
use std::future::Future;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use moka::future::Cache;

/// Tells a unique-constraint violation apart from a genuine server fault.
///
/// A schema-level uniqueness conflict is a caller asking for a value that is
/// taken, not a failure of the server. Matching on the SQLSTATE rather than the
/// message keeps this independent of the constraint's name and of the server's
/// locale.
pub fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    matches!(
        error.sql_err(),
        Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
    )
}

pub const DEFAULT_CACHE_CAPACITY: u64 = 10_000;
pub const LONGEST_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    Keep,
    Share,
    Own,
}

#[derive(Debug)]
pub struct TtlCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    entries: Option<Cache<K, V>>,
}

enum Unkept<V, E> {
    Shared(V),
    Private(V),
    Failure(E),
}

impl<K, V> TtlCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(ttl: Duration) -> Self {
        Self::with_capacity(ttl, DEFAULT_CACHE_CAPACITY)
    }

    pub fn with_capacity(ttl: Duration, capacity: u64) -> Self {
        let entries = (!ttl.is_zero()).then(|| {
            Cache::builder()
                .max_capacity(capacity)
                .time_to_live(ttl.min(LONGEST_CACHE_TTL))
                .build()
        });

        Self { entries }
    }

    pub async fn load<Q, E, F, Fut>(&self, key: &Q, load: F) -> Result<V, E>
    where
        K: Borrow<Q>,
        Q: ToOwned<Owned = K> + Hash + Eq + Sync + ?Sized,
        E: Clone + Send + Sync + 'static,
        F: Fn() -> Fut,
        Fut: Future<Output = Result<(V, Retention), E>>,
    {
        self.load_while(key, || true, load).await
    }

    pub async fn load_while<Q, E, F, Fut, G>(
        &self,
        key: &Q,
        still_current: G,
        load: F,
    ) -> Result<V, E>
    where
        K: Borrow<Q>,
        Q: ToOwned<Owned = K> + Hash + Eq + Sync + ?Sized,
        E: Clone + Send + Sync + 'static,
        F: Fn() -> Fut,
        Fut: Future<Output = Result<(V, Retention), E>>,
        G: Fn() -> bool,
    {
        let Some(entries) = &self.entries else {
            return load().await.map(|(value, _)| value);
        };

        if let Some(value) = entries.get(key).await {
            return Ok(value);
        }

        let loaded = Box::pin(coalesced_load(entries, key, &load)).await;
        if !still_current() {
            entries.invalidate(key).await;
        }

        loaded
    }

    pub fn clear(&self) {
        if let Some(entries) = &self.entries {
            entries.invalidate_all();
        }
    }
}

async fn coalesced_load<K, V, Q, E, F, Fut>(
    entries: &Cache<K, V>,
    key: &Q,
    load: &F,
) -> Result<V, E>
where
    K: Hash + Eq + Send + Sync + Borrow<Q> + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ToOwned<Owned = K> + Hash + Eq + Sync + ?Sized,
    E: Clone + Send + Sync + 'static,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(V, Retention), E>>,
{
    let led = AtomicBool::new(false);
    let loaded = entries
        .try_get_with_by_ref(key, async {
            led.store(true, Ordering::SeqCst);
            match load().await {
                Ok((value, Retention::Keep)) => Ok(value),
                Ok((value, Retention::Share)) => Err(Unkept::Shared(value)),
                Ok((value, Retention::Own)) => Err(Unkept::Private(value)),
                Err(error) => Err(Unkept::Failure(error)),
            }
        })
        .await;

    match loaded {
        Ok(value) => Ok(value),
        Err(unkept) => match &*unkept {
            Unkept::Shared(value) => Ok(value.clone()),
            Unkept::Private(value) if led.load(Ordering::SeqCst) => Ok(value.clone()),
            Unkept::Private(_) => load().await.map(|(value, _)| value),
            Unkept::Failure(error) => Err(error.clone()),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::{Retention, TtlCache};

    const TTL: Duration = Duration::from_secs(60);

    fn cache() -> TtlCache<String, u32> {
        TtlCache::new(TTL)
    }

    async fn kept(cache: &TtlCache<String, u32>, key: &str) -> Option<u32> {
        cache.entries.as_ref()?.get(key).await
    }

    async fn counted(
        calls: &AtomicUsize,
        value: u32,
        retention: Retention,
    ) -> Result<(u32, Retention), String> {
        calls.fetch_add(1, Ordering::SeqCst);
        for _ in 0..16 {
            tokio::task::yield_now().await;
        }
        Ok((value, retention))
    }

    #[tokio::test]
    async fn a_loaded_value_is_served_without_loading_again() {
        let cache = cache();
        let calls = AtomicUsize::new(0);

        let first = cache
            .load("k", || counted(&calls, 7, Retention::Keep))
            .await;
        let second = cache
            .load("k", || counted(&calls, 8, Retention::Keep))
            .await;

        assert_eq!((first, second), (Ok(7), Ok(7)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(kept(&cache, "k").await, Some(7));
    }

    async fn concurrent_loads(retention: Retention) -> usize {
        let cache = Arc::new(cache());
        let calls = Arc::new(AtomicUsize::new(0));

        let readers: Vec<_> = (0..16)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let calls = Arc::clone(&calls);
                tokio::spawn(async move { cache.load("k", || counted(&calls, 7, retention)).await })
            })
            .collect();

        for reader in readers {
            assert_eq!(reader.await.expect("join"), Ok(7));
        }
        calls.load(Ordering::SeqCst)
    }

    #[tokio::test]
    async fn concurrent_misses_on_one_key_load_once() {
        assert_eq!(concurrent_loads(Retention::Keep).await, 1);
    }

    #[tokio::test]
    async fn a_shared_value_is_handed_to_every_waiting_caller_and_not_kept() {
        assert_eq!(concurrent_loads(Retention::Share).await, 1);

        let cache = cache();
        let calls = AtomicUsize::new(0);
        cache
            .load("k", || counted(&calls, 1, Retention::Share))
            .await
            .expect("load");
        assert_eq!(kept(&cache, "k").await, None);
    }

    #[tokio::test]
    async fn a_private_value_makes_each_waiting_caller_load_for_itself() {
        assert_eq!(concurrent_loads(Retention::Own).await, 16);
    }

    #[tokio::test]
    async fn a_failure_is_returned_and_never_kept() {
        let cache = cache();

        let failed = cache
            .load("k", || async {
                Err::<(u32, Retention), _>("down".to_string())
            })
            .await;
        let recovered = cache
            .load("k", || async { Ok::<_, String>((3, Retention::Keep)) })
            .await;

        assert_eq!(failed, Err("down".to_string()));
        assert_eq!(recovered, Ok(3));
    }

    #[tokio::test]
    async fn a_value_that_stopped_being_current_is_dropped_right_after_it_was_kept() {
        let cache = cache();
        let calls = AtomicUsize::new(0);

        cache
            .load_while("k", || false, || counted(&calls, 1, Retention::Keep))
            .await
            .expect("load");

        assert_eq!(kept(&cache, "k").await, None);
        cache
            .load("k", || counted(&calls, 2, Retention::Keep))
            .await
            .expect("load again");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn clearing_drops_every_entry() {
        let cache = cache();
        let calls = AtomicUsize::new(0);
        cache
            .load("k", || counted(&calls, 1, Retention::Keep))
            .await
            .expect("load");

        cache.clear();

        assert_eq!(kept(&cache, "k").await, None);
        cache
            .load("k", || counted(&calls, 1, Retention::Keep))
            .await
            .expect("load again");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_zero_ttl_keeps_nothing() {
        let cache: TtlCache<String, u32> = TtlCache::new(Duration::ZERO);
        let calls = AtomicUsize::new(0);

        cache
            .load("k", || counted(&calls, 1, Retention::Keep))
            .await
            .expect("load");
        cache
            .load("k", || counted(&calls, 1, Retention::Keep))
            .await
            .expect("load");

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(kept(&cache, "k").await, None);
    }

    #[test]
    fn a_huge_ttl_does_not_stop_the_cache_from_being_built() {
        let _cache: TtlCache<String, u32> = TtlCache::new(Duration::from_secs(u64::MAX));
    }

    #[tokio::test]
    async fn the_cache_never_outgrows_its_capacity() {
        let cache: TtlCache<u32, u32> = TtlCache::with_capacity(TTL, 8);

        for key in 0..200_u32 {
            cache
                .load(
                    &key,
                    || async move { Ok::<_, String>((key, Retention::Keep)) },
                )
                .await
                .expect("load");
        }

        let entries = cache.entries.as_ref().expect("an enabled cache");
        entries.run_pending_tasks().await;
        assert!(entries.entry_count() <= 8, "{}", entries.entry_count());
    }
}
