use axum::http::HeaderMap;
use base64::{Engine, engine::general_purpose};

pub fn try_parse_basic_client_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let prefix = "Basic ";
    if value.len() < prefix.len() || !value[..prefix.len()].eq_ignore_ascii_case(prefix) {
        return None;
    }
    let value = &value[prefix.len()..];

    let decoded = general_purpose::STANDARD.decode(value).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;

    let (client_id, client_secret) = decoded.split_once(':')?;
    Some((client_id.to_string(), client_secret.to_string()))
}

#[cfg(test)]
mod tests {
    use super::try_parse_basic_client_credentials;
    use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};

    fn basic(value: &str) -> HeaderValue {
        HeaderValue::from_str(value).unwrap()
    }

    #[test]
    fn parses_basic_credentials() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Basic Y2xpZW50OnNlY3JldA==")); // client:secret

        let creds = try_parse_basic_client_credentials(&headers);
        assert_eq!(creds, Some(("client".to_string(), "secret".to_string())));
    }

    #[test]
    fn rejects_non_basic_header() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Bearer token"));
        assert_eq!(try_parse_basic_client_credentials(&headers), None);
    }

    #[test]
    fn rejects_malformed_base64() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Basic ???"));
        assert_eq!(try_parse_basic_client_credentials(&headers), None);
    }
}
