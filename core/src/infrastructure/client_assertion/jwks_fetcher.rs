use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ferriskey_webhook::endpoint::PrivateEndpoints;
use jsonwebtoken::jwk::JwkSet;
use reqwest::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH};
use reqwest::{Client, StatusCode, Url, redirect};
use tokio::net::lookup_host;

use crate::domain::authentication::client_assertion::entities::{
    ClientAssertionError, JWKS_FETCH_TIMEOUT, MAX_JWKS_BYTES, MAX_JWKS_CACHE_ENTRIES,
    MIN_JWKS_REFETCH_INTERVAL,
};
use crate::domain::authentication::client_assertion::ports::ClientJwksFetcher;
use crate::infrastructure::client_metadata::fetcher::cache_max_age;
use crate::infrastructure::identity_provider::repositories::oauth_client::permitted_addresses;

const USER_AGENT: &str = concat!("FerrisKey/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedJwks {
    pub keys: JwkSet,
    pub max_age: Duration,
}

/// Fetches a client key set over https, pinned to the addresses the host
/// resolved to once those passed the outbound address guard, without
/// following redirects and with a size cap.
#[derive(Debug, Clone)]
pub struct ReqwestClientJwksFetcher {
    private_endpoints: PrivateEndpoints,
    timeout: Duration,
}

impl ReqwestClientJwksFetcher {
    pub fn new(private_endpoints: PrivateEndpoints) -> Self {
        Self {
            private_endpoints,
            timeout: JWKS_FETCH_TIMEOUT,
        }
    }

    async fn pinned_client(&self, url: &Url) -> Result<Client, ClientAssertionError> {
        let host = url
            .host_str()
            .ok_or(ClientAssertionError::KeysUnavailable("the url has no host"))?;
        let port = url
            .port_or_known_default()
            .ok_or(ClientAssertionError::KeysUnavailable("the url has no port"))?;

        let resolved = lookup_host((host, port))
            .await
            .map_err(|_| ClientAssertionError::KeysUnavailable("the host did not resolve"))?;

        let addresses =
            permitted_addresses(resolved, url.scheme() == "https", self.private_endpoints)
                .map_err(|_| {
                    ClientAssertionError::KeysUnavailable(
                        "the host resolves to a forbidden address",
                    )
                })?;

        Client::builder()
            .no_proxy()
            .user_agent(USER_AGENT)
            .timeout(self.timeout)
            .connect_timeout(self.timeout)
            .redirect(redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| {
                ClientAssertionError::KeysUnavailable("the http client could not be built")
            })
    }

    pub async fn fetch_fresh(&self, raw_url: &str) -> Result<FetchedJwks, ClientAssertionError> {
        let url = Url::parse(raw_url)
            .map_err(|_| ClientAssertionError::KeysUnavailable("the jwks_uri is not a url"))?;
        if !matches!(url.scheme(), "https" | "http") {
            return Err(ClientAssertionError::KeysUnavailable(
                "the jwks_uri is not http(s)",
            ));
        }
        let client = self.pinned_client(&url).await?;

        let mut response = client
            .get(url)
            .header(ACCEPT, "application/json, application/jwk-set+json")
            .send()
            .await
            .map_err(|_| ClientAssertionError::KeysUnavailable("the request failed"))?;

        if response.status() != StatusCode::OK {
            return Err(ClientAssertionError::KeysUnavailable(
                "the response was not a 200",
            ));
        }

        let declared_length = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok());
        if declared_length.is_some_and(|length| length > MAX_JWKS_BYTES) {
            return Err(ClientAssertionError::KeysUnavailable(
                "the key set is too large",
            ));
        }

        let max_age = cache_max_age(
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
        );

        let mut body: Vec<u8> = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| ClientAssertionError::KeysUnavailable("the body could not be read"))?
        {
            if body.len() + chunk.len() > MAX_JWKS_BYTES {
                return Err(ClientAssertionError::KeysUnavailable(
                    "the key set is too large",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let keys: JwkSet = serde_json::from_slice(&body)
            .map_err(|_| ClientAssertionError::KeysUnavailable("the body is not a key set"))?;

        Ok(FetchedJwks { keys, max_age })
    }
}

#[derive(Debug)]
struct Entry {
    keys: JwkSet,
    fetched_at: Instant,
    expires_at: Instant,
}

/// Caches key sets per `jwks_uri` for as long as the publisher allows, and
/// refetches at most once every [`MIN_JWKS_REFETCH_INTERVAL`] on a refresh, so
/// assertions with made-up `kid`s cannot turn FerrisKey into a request
/// amplifier against the client's host.
#[derive(Debug)]
pub struct CachedClientJwksFetcher {
    inner: ReqwestClientJwksFetcher,
    entries: Mutex<HashMap<String, Entry>>,
    capacity: usize,
}

impl CachedClientJwksFetcher {
    pub fn new(inner: ReqwestClientJwksFetcher) -> Self {
        Self {
            inner,
            entries: Mutex::new(HashMap::new()),
            capacity: MAX_JWKS_CACHE_ENTRIES,
        }
    }

    fn cached(&self, uri: &str, refresh: bool, now: Instant) -> Option<JwkSet> {
        let entries = self.entries.lock().ok()?;
        let entry = entries.get(uri)?;
        let fresh = entry.expires_at > now;
        let refetched_recently = now.duration_since(entry.fetched_at) < MIN_JWKS_REFETCH_INTERVAL;
        let usable = if refresh { refetched_recently } else { fresh };
        usable.then(|| entry.keys.clone())
    }

    fn store(&self, uri: &str, fetched: &FetchedJwks, now: Instant) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };

        if entries.len() >= self.capacity && !entries.contains_key(uri) {
            entries.retain(|_, entry| entry.expires_at > now);
        }
        if entries.len() >= self.capacity && !entries.contains_key(uri) {
            let oldest = entries
                .iter()
                .min_by_key(|(_, entry)| entry.fetched_at)
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                entries.remove(&key);
            }
        }

        entries.insert(
            uri.to_string(),
            Entry {
                keys: fetched.keys.clone(),
                fetched_at: now,
                expires_at: now + fetched.max_age,
            },
        );
    }
}

impl ClientJwksFetcher for CachedClientJwksFetcher {
    async fn fetch(&self, uri: &str, refresh: bool) -> Result<JwkSet, ClientAssertionError> {
        if let Some(keys) = self.cached(uri, refresh, Instant::now()) {
            return Ok(keys);
        }

        let fetched = self.inner.fetch_fresh(uri).await?;
        self.store(uri, &fetched, Instant::now());
        Ok(fetched.keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> JwkSet {
        JwkSet { keys: vec![] }
    }

    fn cache() -> CachedClientJwksFetcher {
        CachedClientJwksFetcher::new(ReqwestClientJwksFetcher::new(PrivateEndpoints::Forbidden))
    }

    const URI: &str = "https://app.example/jwks.json";

    #[test]
    fn a_fresh_entry_answers_a_normal_lookup() {
        let cache = cache();
        let now = Instant::now();
        cache.store(
            URI,
            &FetchedJwks {
                keys: set(),
                max_age: Duration::from_secs(300),
            },
            now,
        );

        assert!(
            cache
                .cached(URI, false, now + Duration::from_secs(299))
                .is_some()
        );
        assert!(
            cache
                .cached(URI, false, now + Duration::from_secs(301))
                .is_none()
        );
    }

    #[test]
    fn a_refresh_bypasses_the_cache_once_the_refetch_interval_passed() {
        let cache = cache();
        let now = Instant::now();
        cache.store(
            URI,
            &FetchedJwks {
                keys: set(),
                max_age: Duration::from_secs(300),
            },
            now,
        );

        assert!(
            cache
                .cached(URI, true, now + Duration::from_secs(1))
                .is_some()
        );
        assert!(
            cache
                .cached(
                    URI,
                    true,
                    now + MIN_JWKS_REFETCH_INTERVAL + Duration::from_secs(1)
                )
                .is_none()
        );
    }

    #[tokio::test]
    async fn forbidden_targets_are_refused_without_the_test_flag() {
        let fetcher = ReqwestClientJwksFetcher::new(PrivateEndpoints::Forbidden);
        for url in [
            "https://127.0.0.1:9/jwks.json",
            "https://169.254.169.254/jwks.json",
            "https://10.0.0.1/jwks.json",
            "http://example.com/jwks.json",
            "file:///etc/passwd",
        ] {
            assert!(
                matches!(
                    fetcher.fetch_fresh(url).await,
                    Err(ClientAssertionError::KeysUnavailable(_))
                ),
                "{url}"
            );
        }
    }
}
