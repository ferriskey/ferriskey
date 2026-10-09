use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::domain::client_metadata::entities::{
    ClientMetadataError, FAILURE_CACHE_AGE, FetchedClientMetadata, MAX_CACHE_ENTRIES,
};
use crate::domain::client_metadata::ports::ClientMetadataDocumentFetcher;

type Outcome = Result<FetchedClientMetadata, ClientMetadataError>;

#[derive(Debug)]
struct Entry {
    outcome: Outcome,
    expires_at: Instant,
}

#[derive(Debug)]
pub struct CachedClientMetadataFetcher<F>
where
    F: ClientMetadataDocumentFetcher,
{
    inner: F,
    entries: Mutex<HashMap<String, Entry>>,
    capacity: usize,
}

impl<F> CachedClientMetadataFetcher<F>
where
    F: ClientMetadataDocumentFetcher,
{
    pub fn new(inner: F) -> Self {
        Self::with_capacity(inner, MAX_CACHE_ENTRIES)
    }

    pub fn with_capacity(inner: F, capacity: usize) -> Self {
        Self {
            inner,
            entries: Mutex::new(HashMap::new()),
            capacity,
        }
    }

    fn lookup(&self, url: &str, now: Instant) -> Option<Outcome> {
        let entries = self.entries.lock().ok()?;
        entries
            .get(url)
            .filter(|entry| entry.expires_at > now)
            .map(|entry| entry.outcome.clone())
    }

    fn store(&self, url: &str, outcome: &Outcome, now: Instant) {
        let ttl = match outcome {
            Ok(fetched) => fetched.max_age,
            Err(_) => FAILURE_CACHE_AGE,
        };

        if ttl == Duration::ZERO {
            return;
        }

        let Ok(mut entries) = self.entries.lock() else {
            return;
        };

        if entries.len() >= self.capacity && !entries.contains_key(url) {
            entries.retain(|_, entry| entry.expires_at > now);
        }

        if entries.len() >= self.capacity && !entries.contains_key(url) {
            let oldest = entries
                .iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                entries.remove(&key);
            }
        }

        entries.insert(
            url.to_string(),
            Entry {
                outcome: outcome.clone(),
                expires_at: now + ttl,
            },
        );
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries
            .lock()
            .map(|entries| entries.len())
            .unwrap_or(0)
    }
}

impl<F> ClientMetadataDocumentFetcher for CachedClientMetadataFetcher<F>
where
    F: ClientMetadataDocumentFetcher,
{
    async fn fetch(&self, url: &str) -> Outcome {
        if let Some(outcome) = self.lookup(url, Instant::now()) {
            return outcome;
        }

        let outcome = self.inner.fetch(url).await;
        self.store(url, &outcome, Instant::now());
        outcome
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::domain::client_metadata::entities::ClientMetadataDocument;

    #[derive(Debug, Clone)]
    struct CountingFetcher {
        calls: Arc<AtomicUsize>,
        outcome: Outcome,
    }

    impl ClientMetadataDocumentFetcher for CountingFetcher {
        async fn fetch(&self, _url: &str) -> Outcome {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.outcome.clone()
        }
    }

    fn fetched(max_age: Duration) -> Outcome {
        Ok(FetchedClientMetadata {
            document: ClientMetadataDocument {
                client_id: "https://app.example/c.json".to_string(),
                client_name: None,
                redirect_uris: vec![],
                token_endpoint_auth_method: None,
            },
            max_age,
        })
    }

    fn cached(
        outcome: Outcome,
        capacity: usize,
    ) -> (
        CachedClientMetadataFetcher<CountingFetcher>,
        Arc<AtomicUsize>,
    ) {
        let calls = Arc::new(AtomicUsize::new(0));
        let fetcher = CountingFetcher {
            calls: calls.clone(),
            outcome,
        };
        (
            CachedClientMetadataFetcher::with_capacity(fetcher, capacity),
            calls,
        )
    }

    #[tokio::test]
    async fn a_document_is_fetched_once_within_its_max_age() {
        let (cache, calls) = cached(fetched(Duration::from_secs(60)), 8);

        assert!(cache.fetch("https://a.example/c.json").await.is_ok());
        assert!(cache.fetch("https://a.example/c.json").await.is_ok());

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_zero_max_age_is_never_cached() {
        let (cache, calls) = cached(fetched(Duration::ZERO), 8);

        assert!(cache.fetch("https://a.example/c.json").await.is_ok());
        assert!(cache.fetch("https://a.example/c.json").await.is_ok());

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_failure_is_cached_briefly() {
        let (cache, calls) = cached(Err(ClientMetadataError::FetchFailed("down")), 8);

        assert!(cache.fetch("https://a.example/c.json").await.is_err());
        assert!(cache.fetch("https://a.example/c.json").await.is_err());

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn an_expired_entry_is_not_served() {
        let (cache, _) = cached(fetched(Duration::from_secs(60)), 8);
        let now = Instant::now();

        cache.store(
            "https://a.example/c.json",
            &fetched(Duration::from_secs(60)),
            now,
        );

        assert!(cache.lookup("https://a.example/c.json", now).is_some());
        assert!(
            cache
                .lookup("https://a.example/c.json", now + Duration::from_secs(61))
                .is_none()
        );
    }

    #[tokio::test]
    async fn the_cache_never_grows_past_its_capacity() {
        let (cache, _) = cached(fetched(Duration::from_secs(60)), 2);

        for index in 0..5 {
            let _ = cache
                .fetch(&format!("https://a.example/{index}.json"))
                .await;
        }

        assert_eq!(cache.len(), 2);
    }
}
