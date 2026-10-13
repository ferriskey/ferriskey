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
        webapp_base_url(&state.args, &parts.headers, &parts.uri).map(WebappUrl)
    }
}

pub fn webapp_base_url(args: &Args, headers: &HeaderMap, uri: &Uri) -> Result<String, ApiError> {
    if !args.webapp_url_from_request {
        return Ok(args.webapp_url.trim_end_matches('/').to_string());
    }

    let authority = request_authority(headers, uri)
        .ok_or_else(|| ApiError::BadRequest("Invalid Host header".into()))?;
    let scheme = request_scheme(headers);

    let origin = url::Url::parse(&format!("{scheme}://{authority}"))
        .ok()
        .filter(|url| {
            url.username().is_empty()
                && url.password().is_none()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
        })
        .ok_or_else(|| ApiError::BadRequest("Invalid Host header".into()))?;

    let host = origin
        .host_str()
        .ok_or_else(|| ApiError::BadRequest("Invalid Host header".into()))?;
    let host = host.strip_suffix('.').unwrap_or(host);

    let default_port = if scheme == "https" { 443 } else { 80 };
    let port = origin.port().unwrap_or(default_port);

    let allowed = args
        .webapp_allowed_hosts
        .iter()
        .filter_map(|entry| HostRule::parse(entry))
        .any(|rule| rule.matches(host, port, default_port));

    if !allowed {
        return Err(ApiError::BadRequest("Host is not allowed".into()));
    }

    Ok(match origin.port() {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    })
}

/// The one Host a request names: its Host header, or the URI authority when
/// there is none (HTTP/2). Several Host headers, or one that is not ASCII,
/// name none.
fn request_authority(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    let mut hosts = headers.get_all("host").iter();

    match (hosts.next(), hosts.next()) {
        (Some(host), None) => host.to_str().ok().map(str::to_string),
        (None, _) => uri.authority().map(|authority| authority.to_string()),
        (Some(_), Some(_)) => None,
    }
}

/// Refuses WEBAPP_URL_FROM_REQUEST without a usable WEBAPP_ALLOWED_HOSTS:
/// links would otherwise follow whatever Host a request carries.
pub fn check_webapp_url_settings(args: &Args) -> Result<(), String> {
    if !args.webapp_url_from_request {
        return Ok(());
    }

    let mut entries = args
        .webapp_allowed_hosts
        .iter()
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .peekable();

    if entries.peek().is_none() {
        return Err(
            "WEBAPP_URL_FROM_REQUEST=true requires WEBAPP_ALLOWED_HOSTS: list the hosts the webapp is served on"
                .to_string(),
        );
    }

    match entries.find(|entry| HostRule::parse(entry).is_none()) {
        Some(invalid) => Err(format!(
            "WEBAPP_ALLOWED_HOSTS: `{invalid}` is not a host, host:port or *.domain entry"
        )),
        None => Ok(()),
    }
}

/// One WEBAPP_ALLOWED_HOSTS entry: `host`, `host:port`, or `*.domain` where
/// `*` stands for exactly one label.
#[derive(Debug, PartialEq, Eq)]
struct HostRule {
    wildcard: bool,
    domain: String,
    port: Option<u16>,
}

impl HostRule {
    fn parse(entry: &str) -> Option<Self> {
        let entry = entry.trim().to_ascii_lowercase();

        let (name, port) = match entry.rsplit_once(':') {
            Some((name, port))
                if !name.ends_with(':') && port.bytes().all(|b| b.is_ascii_digit()) =>
            {
                (name.to_string(), Some(port.parse().ok()?))
            }
            _ => (entry, None),
        };

        let name = name.strip_suffix('.').unwrap_or(&name);
        let (wildcard, domain) = match name.strip_prefix("*.") {
            Some(domain) => (true, domain),
            None => (false, name),
        };

        let valid = !domain.is_empty()
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !domain.contains("..")
            && domain
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-.[]:".contains(&b));

        valid.then(|| Self {
            wildcard,
            domain: domain.to_string(),
            port,
        })
    }

    /// `host` is lowercase without a trailing dot. An entry without a port
    /// only matches the scheme's default port.
    fn matches(&self, host: &str, port: u16, default_port: u16) -> bool {
        if self.port.unwrap_or(default_port) != port {
            return false;
        }

        if !self.wildcard {
            return host == self.domain;
        }

        host.strip_suffix(self.domain.as_str())
            .and_then(|rest| rest.strip_suffix('.'))
            .is_some_and(|label| !label.is_empty() && !label.contains('.'))
    }
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
    use super::{
        check_webapp_url_settings, public_base_url, request_base_url, root_scoped_base_url,
        webapp_base_url,
    };
    use crate::{api_entities::api_error::ApiError, args::Args};
    use axum::http::{HeaderMap, HeaderValue, Uri};

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

    fn resolve(args: &Args, headers: &HeaderMap) -> Result<String, ApiError> {
        webapp_base_url(args, headers, &Uri::from_static("/realms/acme/protocol"))
    }

    fn refused(result: Result<String, ApiError>) -> bool {
        matches!(result, Err(ApiError::BadRequest(_)))
    }

    #[test]
    fn the_configured_webapp_url_is_used_when_the_flag_is_off() {
        let webapp = resolve(
            &args(false, &[]),
            &headers("acme.example.com", Some("https")),
        );

        assert_eq!(webapp.unwrap(), "https://console.example.com");
    }

    #[test]
    fn without_the_flag_a_missing_host_changes_nothing() {
        assert_eq!(
            resolve(&args(false, &[]), &HeaderMap::new()).unwrap(),
            "https://console.example.com"
        );
    }

    #[test]
    fn the_webapp_url_follows_the_request_host_when_the_flag_is_on() {
        let args = args(true, &["acme.example.com", "beta.example.com"]);

        assert_eq!(
            resolve(&args, &headers("acme.example.com", Some("https"))).unwrap(),
            "https://acme.example.com"
        );
        assert_eq!(
            resolve(&args, &headers("beta.example.com", Some("https"))).unwrap(),
            "https://beta.example.com"
        );
    }

    #[test]
    fn the_webapp_scheme_is_http_unless_the_proxy_says_https() {
        let args = args(true, &["acme.localhost:8080"]);

        assert_eq!(
            resolve(&args, &headers("acme.localhost:8080", None)).unwrap(),
            "http://acme.localhost:8080"
        );
        assert_eq!(
            resolve(&args, &headers("acme.localhost:8080", Some("http"))).unwrap(),
            "http://acme.localhost:8080"
        );
    }

    #[test]
    fn the_webapp_url_matches_the_issuer_origin_without_the_root_path() {
        let mut args = args(true, &["acme.example.com"]);
        args.server.root_path = "/api".to_string();
        let headers = headers("acme.example.com", Some("https"));

        let issuer_base = root_scoped_base_url(&request_base_url(&headers), &args.server.root_path);
        let webapp = resolve(&args, &headers).unwrap();

        assert_eq!(issuer_base, "https://acme.example.com/api");
        assert_eq!(webapp, "https://acme.example.com");
    }

    #[test]
    fn a_host_outside_the_allowlist_is_refused() {
        let args = args(true, &["acme.example.com", " Beta.Example.com "]);

        assert_eq!(
            resolve(&args, &headers("beta.example.com", Some("https"))).unwrap(),
            "https://beta.example.com"
        );
        assert!(refused(resolve(
            &args,
            &headers("evil.example.net", Some("https"))
        )));
    }

    #[test]
    fn an_empty_allowlist_refuses_every_host() {
        assert!(refused(resolve(
            &args(true, &[]),
            &headers("acme.example.com", Some("https"))
        )));
    }

    #[test]
    fn a_wildcard_matches_exactly_one_label() {
        let args = args(true, &["*.example.com"]);
        let host = |host| resolve(&args, &headers(host, Some("https")));

        assert_eq!(
            host("acme.example.com").unwrap(),
            "https://acme.example.com"
        );
        assert!(refused(host("example.com")));
        assert!(refused(host("a.b.example.com")));
        assert!(refused(host("acmeexample.com")));
        assert!(refused(host("acme.example.com.evil.net")));
    }

    #[test]
    fn hosts_are_compared_lowercase_and_without_one_trailing_dot() {
        let args = args(true, &["*.Example.COM.", "beta.example.com"]);

        assert_eq!(
            resolve(&args, &headers("ACME.example.com.", Some("https"))).unwrap(),
            "https://acme.example.com"
        );
        assert_eq!(
            resolve(&args, &headers("beta.example.com.", Some("https"))).unwrap(),
            "https://beta.example.com"
        );
    }

    #[test]
    fn an_entry_without_port_only_matches_the_default_port() {
        let args = args(true, &["acme.example.com"]);

        assert_eq!(
            resolve(&args, &headers("acme.example.com:443", Some("https"))).unwrap(),
            "https://acme.example.com"
        );
        assert!(refused(resolve(
            &args,
            &headers("acme.example.com:8443", Some("https"))
        )));
    }

    #[test]
    fn an_entry_with_a_port_only_matches_that_port() {
        let args = args(true, &["*.example.com:8443"]);

        assert_eq!(
            resolve(&args, &headers("acme.example.com:8443", Some("https"))).unwrap(),
            "https://acme.example.com:8443"
        );
        assert!(refused(resolve(
            &args,
            &headers("acme.example.com", Some("https"))
        )));
    }

    #[test]
    fn a_host_that_would_change_the_link_target_is_refused() {
        let args = args(true, &["*.example.com", "evil.net"]);

        for host in [
            "acme.example.com@evil.net",
            "evil.net/acme",
            "evil.net?x",
            "evil.net#x",
        ] {
            assert!(
                refused(resolve(&args, &headers(host, Some("https")))),
                "{host} was accepted"
            );
        }
    }

    #[test]
    fn a_missing_host_is_refused_when_the_flag_is_on() {
        let args = args(true, &["acme.example.com"]);

        assert!(refused(resolve(&args, &HeaderMap::new())));
    }

    #[test]
    fn a_non_ascii_host_is_refused() {
        let args = args(true, &["*.example.com"]);
        let mut headers = HeaderMap::new();
        headers.insert(
            "host",
            HeaderValue::from_bytes("é.example.com".as_bytes()).unwrap(),
        );

        assert!(refused(resolve(&args, &headers)));
    }

    #[test]
    fn several_host_headers_are_refused() {
        let args = args(true, &["acme.example.com", "beta.example.com"]);
        let mut headers = headers("acme.example.com", Some("https"));
        headers.append("host", HeaderValue::from_static("beta.example.com"));

        assert!(refused(resolve(&args, &headers)));
    }

    #[test]
    fn without_a_host_header_the_uri_authority_is_used() {
        let args = args(true, &["acme.example.com"]);
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        let uri = Uri::from_static("https://acme.example.com/realms/acme/protocol");

        assert_eq!(
            webapp_base_url(&args, &headers, &uri).unwrap(),
            "https://acme.example.com"
        );
    }

    #[test]
    fn the_flag_without_an_allowlist_refuses_to_start() {
        assert!(check_webapp_url_settings(&args(true, &[])).is_err());
        assert!(check_webapp_url_settings(&args(true, &[" ", ""])).is_err());
        assert!(check_webapp_url_settings(&args(true, &["*.example.com"])).is_ok());
        assert!(check_webapp_url_settings(&args(false, &[])).is_ok());
    }

    #[test]
    fn a_malformed_allowlist_entry_refuses_to_start() {
        for entry in [
            "*",
            "*.",
            "a.*.example.com",
            "evil.net/x",
            "acme.example.com:99999",
        ] {
            assert!(
                check_webapp_url_settings(&args(true, &[entry])).is_err(),
                "{entry} was accepted"
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
