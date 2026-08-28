use axum::{RequestPartsExt, extract::FromRequestParts, http::Uri};

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

        let headers = &parts.headers;

        let scheme = if headers
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok())
            .map(|s| s == "https")
            .unwrap_or(false)
        {
            "https"
        } else {
            "http"
        };

        let host = headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        let base_url = format!("{scheme}://{host}");

        let full_url = uri.to_string();

        Ok(FullUrl(full_url, base_url))
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
    use super::public_base_url;

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
