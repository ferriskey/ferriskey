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

        let base_url = format!("{}://{}", scheme, host);

        let full_url = uri.to_string();

        Ok(FullUrl(full_url, base_url))
    }
}
