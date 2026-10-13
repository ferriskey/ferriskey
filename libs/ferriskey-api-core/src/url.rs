use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{HeaderMap, Uri},
};

use crate::{api_entities::api_error::ApiError, app_state::AppState, args::Args};

#[derive(Debug, Clone)]
pub struct FullUrl(pub String, pub String);

impl<S> FromRequestParts<S> for FullUrl
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let uri = match parts.extract::<Uri>().await {
            Ok(uri) => uri,
            Err(_) => {
                let response = axum::response::Response::builder()
                    .status(axum::http::StatusCode::BAD_REQUEST)
                    .body("Invalid URI".into())
                    .unwrap_or_default();
                return Err(response);
            }
        };

        let base_url = request_base_url(&parts.headers);

        let full_url = uri.to_string();

        Ok(FullUrl(full_url, base_url))
    }
}

fn request_scheme(headers: &HeaderMap) -> &'static str {
    let forwarded_https = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|s| s == "https");

    if forwarded_https { "https" } else { "http" }
}

fn request_host(headers: &HeaderMap) -> &str {
    headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
}

/// The `scheme://host` a request was addressed to, without the root path.
pub fn request_base_url(headers: &HeaderMap) -> String {
    format!("{}://{}", request_scheme(headers), request_host(headers))
}

/// Origin of the webapp (login and consent pages, emailed links, WebAuthn
/// origin), without a trailing slash: WEBAPP_URL, or the request's own origin
/// when WEBAPP_URL_FROM_REQUEST is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebappUrl(pub String);

impl FromRequestParts<AppState> for WebappUrl {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        webapp_base_url(&state.args, &parts.headers).map(WebappUrl)
    }
}

pub fn webapp_base_url(args: &Args, headers: &HeaderMap) -> Result<String, ApiError> {
    if !args.webapp_url_from_request {
        return Ok(args.webapp_url.trim_end_matches('/').to_string());
    }

    let host = request_host(headers);

    let mut allowed = args
        .webapp_allowed_hosts
        .iter()
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .peekable();

    if allowed.peek().is_some() && !allowed.any(|entry| entry.eq_ignore_ascii_case(host)) {
        return Err(ApiError::BadRequest("Host is not allowed".into()));
    }

    let origin = url::Url::parse(&format!("{}://{host}", request_scheme(headers)))
        .ok()
        .filter(|url| {
            url.username().is_empty()
                && url.password().is_none()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
        })
        .ok_or_else(|| ApiError::BadRequest("Invalid Host header".into()))?;

    Ok(origin.origin().ascii_serialization())
}

pub fn public_base_url(configured: Option<&str>, request_base_url: &str) -> String {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(request_base_url)
        .trim_end_matches('/')
        .to_string()
}

/// Joins a base URL with an optional root path segment.
pub fn root_scoped_base_url(base_url: &str, root_path: &str) -> String {
    if root_path.is_empty() || root_path == "/" {
        return base_url.to_string();
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        root_path.trim_start_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::{public_base_url, request_base_url, root_scoped_base_url, webapp_base_url};
    use crate::{api_entities::api_error::ApiError, args::Args};
    use axum::http::{HeaderMap, HeaderValue};

    fn headers(host: &str, forwarded_proto: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_str(host).unwrap());
        if let Some(proto) = forwarded_proto {
            headers.insert("x-forwarded-proto", HeaderValue::from_str(proto).unwrap());
        }
        headers
    }

    fn args(from_request: bool, allowed_hosts: &[&str]) -> Args {
        Args {
            webapp_url: "https://console.example.com/".to_string(),
            webapp_url_from_request: from_request,
            webapp_allowed_hosts: allowed_hosts.iter().map(|h| h.to_string()).collect(),
            ..Args::default()
        }
    }

    #[test]
    fn the_configured_webapp_url_is_used_when_the_flag_is_off() {
        let webapp = webapp_base_url(
            &args(false, &[]),
            &headers("acme.example.com", Some("https")),
        );

        assert_eq!(webapp.unwrap(), "https://console.example.com");
    }

    #[test]
    fn the_webapp_url_follows_the_request_host_when_the_flag_is_on() {
        let args = args(true, &[]);

        assert_eq!(
            webapp_base_url(&args, &headers("acme.example.com", Some("https"))).unwrap(),
            "https://acme.example.com"
        );
        assert_eq!(
            webapp_base_url(&args, &headers("beta.example.com", Some("https"))).unwrap(),
            "https://beta.example.com"
        );
    }

    #[test]
    fn the_webapp_scheme_is_http_unless_the_proxy_says_https() {
        let args = args(true, &[]);

        assert_eq!(
            webapp_base_url(&args, &headers("acme.localhost:8080", None)).unwrap(),
            "http://acme.localhost:8080"
        );
        assert_eq!(
            webapp_base_url(&args, &headers("acme.localhost:8080", Some("http"))).unwrap(),
            "http://acme.localhost:8080"
        );
    }

    #[test]
    fn the_webapp_url_matches_the_issuer_origin_without_the_root_path() {
        let mut args = args(true, &[]);
        args.server.root_path = "/api".to_string();
        let headers = headers("acme.example.com", Some("https"));

        let issuer_base = root_scoped_base_url(&request_base_url(&headers), &args.server.root_path);
        let webapp = webapp_base_url(&args, &headers).unwrap();

        assert_eq!(issuer_base, "https://acme.example.com/api");
        assert_eq!(webapp, "https://acme.example.com");
    }

    #[test]
    fn a_host_outside_the_allowlist_is_refused() {
        let args = args(true, &["acme.example.com", " Beta.Example.com "]);

        assert_eq!(
            webapp_base_url(&args, &headers("beta.example.com", Some("https"))).unwrap(),
            "https://beta.example.com"
        );
        assert!(matches!(
            webapp_base_url(&args, &headers("evil.example.net", Some("https"))),
            Err(ApiError::BadRequest(_))
        ));
    }

    #[test]
    fn a_host_that_would_change_the_link_target_is_refused() {
        let args = args(true, &[]);

        for host in [
            "acme.example.com@evil.net",
            "evil.net/acme",
            "evil.net?x",
            "evil.net#x",
        ] {
            assert!(
                matches!(
                    webapp_base_url(&args, &headers(host, Some("https"))),
                    Err(ApiError::BadRequest(_))
                ),
                "{host} was accepted"
            );
        }
    }

    #[test]
    fn the_request_host_is_used_when_no_public_url_is_configured() {
        assert_eq!(
            public_base_url(None, "https://node-7.internal:3333"),
            "https://node-7.internal:3333"
        );
    }

    #[test]
    fn a_configured_public_url_wins_over_the_request_host() {
        assert_eq!(
            public_base_url(
                Some("https://auth.example.com"),
                "https://node-7.internal:3333"
            ),
            "https://auth.example.com"
        );
    }

    #[test]
    fn a_configured_public_url_is_stable_whatever_the_request_carries() {
        let configured = Some("https://auth.example.com");
        assert_eq!(
            public_base_url(configured, "http://10.0.0.4"),
            public_base_url(configured, "https://some-other-name")
        );
    }

    #[test]
    fn a_trailing_slash_is_dropped_so_joins_never_double_up() {
        assert_eq!(
            public_base_url(Some("https://auth.example.com/"), "http://ignored"),
            "https://auth.example.com"
        );
    }

    #[test]
    fn a_blank_configured_value_falls_back_to_the_request_host() {
        assert_eq!(
            public_base_url(Some("   "), "https://node-7.internal"),
            "https://node-7.internal"
        );
    }
}
