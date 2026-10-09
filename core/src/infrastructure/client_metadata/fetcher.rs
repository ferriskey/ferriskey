use std::time::Duration;

use ferriskey_webhook::endpoint::PrivateEndpoints;
use reqwest::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, StatusCode, Url, redirect};
use tokio::net::lookup_host;

use crate::domain::client_metadata::entities::{
    ClientMetadataDocument, ClientMetadataError, DEFAULT_CACHE_AGE, FETCH_TIMEOUT,
    FetchedClientMetadata, MAX_CACHE_AGE, MAX_DOCUMENT_BYTES,
};
use crate::domain::client_metadata::ports::ClientMetadataDocumentFetcher;
use crate::infrastructure::identity_provider::repositories::oauth_client::permitted_addresses;

const USER_AGENT: &str = concat!("FerrisKey/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct ReqwestClientMetadataFetcher {
    private_endpoints: PrivateEndpoints,
    timeout: Duration,
}

impl ReqwestClientMetadataFetcher {
    pub fn new(private_endpoints: PrivateEndpoints) -> Self {
        Self {
            private_endpoints,
            timeout: FETCH_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn pinned_client(&self, url: &Url) -> Result<Client, ClientMetadataError> {
        let host = url
            .host_str()
            .ok_or(ClientMetadataError::FetchFailed("the url has no host"))?;
        let port = url
            .port_or_known_default()
            .ok_or(ClientMetadataError::FetchFailed("the url has no port"))?;

        let resolved = lookup_host((host, port))
            .await
            .map_err(|_| ClientMetadataError::FetchFailed("the host did not resolve"))?;

        let addresses =
            permitted_addresses(resolved, url.scheme() == "https", self.private_endpoints)
                .map_err(|_| {
                    ClientMetadataError::FetchFailed("the host resolves to a forbidden address")
                })?;

        Client::builder()
            .no_proxy()
            .user_agent(USER_AGENT)
            .timeout(self.timeout)
            .connect_timeout(self.timeout)
            .redirect(redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| ClientMetadataError::FetchFailed("the http client could not be built"))
    }
}

impl ClientMetadataDocumentFetcher for ReqwestClientMetadataFetcher {
    async fn fetch(&self, raw_url: &str) -> Result<FetchedClientMetadata, ClientMetadataError> {
        let url = Url::parse(raw_url).map_err(|_| ClientMetadataError::InvalidUrl)?;
        let client = self.pinned_client(&url).await?;

        let mut response = client
            .get(url)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| ClientMetadataError::FetchFailed("the request failed"))?;

        if response.status() != StatusCode::OK {
            return Err(ClientMetadataError::FetchFailed(
                "the response was not a 200",
            ));
        }

        if !is_json(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
        ) {
            return Err(ClientMetadataError::FetchFailed(
                "the response is not application/json",
            ));
        }

        let declared_length = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok());

        if declared_length.is_some_and(|length| length > MAX_DOCUMENT_BYTES) {
            return Err(ClientMetadataError::FetchFailed(
                "the document is too large",
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
            .map_err(|_| ClientMetadataError::FetchFailed("the body could not be read"))?
        {
            if body.len() + chunk.len() > MAX_DOCUMENT_BYTES {
                return Err(ClientMetadataError::FetchFailed(
                    "the document is too large",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let document: ClientMetadataDocument = serde_json::from_slice(&body).map_err(|_| {
            ClientMetadataError::InvalidDocument("the body is not a client document")
        })?;

        Ok(FetchedClientMetadata { document, max_age })
    }
}

fn is_json(content_type: Option<&str>) -> bool {
    let Some(content_type) = content_type else {
        return false;
    };

    let essence = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    essence == "application/json" || essence.ends_with("+json")
}

pub(crate) fn cache_max_age(header: Option<&str>) -> Duration {
    let Some(header) = header else {
        return DEFAULT_CACHE_AGE;
    };

    let mut max_age = None;
    for directive in header.split(',').map(|d| d.trim().to_ascii_lowercase()) {
        if directive == "no-store" || directive == "no-cache" {
            return Duration::ZERO;
        }
        if let Some(value) = directive.strip_prefix("max-age=") {
            max_age = value.trim_matches('"').parse::<u64>().ok();
        }
    }

    max_age
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_CACHE_AGE)
        .min(MAX_CACHE_AGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_content_types_are_accepted() {
        assert!(is_json(Some("application/json")));
        assert!(is_json(Some("Application/JSON; charset=utf-8")));
        assert!(is_json(Some("application/client+json")));
    }

    #[test]
    fn other_content_types_are_refused() {
        assert!(!is_json(None));
        assert!(!is_json(Some("text/html")));
        assert!(!is_json(Some("text/plain; application/json")));
    }

    #[test]
    fn max_age_is_honoured_and_capped_at_a_day() {
        assert_eq!(
            cache_max_age(Some("public, max-age=120")),
            Duration::from_secs(120)
        );
        assert_eq!(cache_max_age(Some("max-age=999999999")), MAX_CACHE_AGE);
    }

    #[test]
    fn missing_or_unreadable_directives_fall_back_to_the_default() {
        assert_eq!(cache_max_age(None), DEFAULT_CACHE_AGE);
        assert_eq!(cache_max_age(Some("public")), DEFAULT_CACHE_AGE);
        assert_eq!(cache_max_age(Some("max-age=abc")), DEFAULT_CACHE_AGE);
    }

    #[test]
    fn no_store_and_no_cache_disable_caching() {
        assert_eq!(cache_max_age(Some("no-store")), Duration::ZERO);
        assert_eq!(cache_max_age(Some("max-age=60, no-cache")), Duration::ZERO);
    }

    #[tokio::test]
    async fn a_loopback_target_is_refused_without_the_test_flag() {
        let fetcher = ReqwestClientMetadataFetcher::new(PrivateEndpoints::Forbidden);
        let error = fetcher
            .fetch("https://127.0.0.1:9/client.json")
            .await
            .expect_err("loopback must be refused");

        assert!(matches!(error, ClientMetadataError::FetchFailed(_)));
    }

    #[tokio::test]
    async fn private_and_link_local_literals_are_refused_without_the_test_flag() {
        let fetcher = ReqwestClientMetadataFetcher::new(PrivateEndpoints::Forbidden);
        for host in [
            "10.0.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "[::1]",
            "0.0.0.0",
        ] {
            let error = fetcher
                .fetch(&format!("https://{host}/client.json"))
                .await
                .expect_err("forbidden address must be refused");
            assert!(
                matches!(error, ClientMetadataError::FetchFailed(_)),
                "{host}"
            );
        }
    }

    #[tokio::test]
    async fn cleartext_http_is_refused_without_the_test_flag() {
        let fetcher = ReqwestClientMetadataFetcher::new(PrivateEndpoints::Forbidden);
        let error = fetcher
            .fetch("http://example.com/client.json")
            .await
            .expect_err("cleartext must be refused");

        assert!(matches!(error, ClientMetadataError::FetchFailed(_)));
    }
}
