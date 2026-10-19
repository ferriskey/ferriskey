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
    Some((form_decoded(client_id)?, form_decoded(client_secret)?))
}

fn form_decoded(part: &str) -> Option<String> {
    urlencoding::decode(&part.replace('+', " "))
        .ok()
        .map(|decoded| decoded.into_owned())
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

    fn encoded(credentials: &str) -> HeaderMap {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            basic(&format!("Basic {}", STANDARD.encode(credentials))),
        );
        headers
    }

    #[test]
    fn a_secret_containing_a_colon_is_kept_whole() {
        assert_eq!(
            try_parse_basic_client_credentials(&encoded("client:se:cr:et")),
            Some(("client".to_string(), "se:cr:et".to_string()))
        );
    }

    #[test]
    fn form_encoded_parts_are_decoded_as_rfc_6749_requires() {
        assert_eq!(
            try_parse_basic_client_credentials(&encoded("my%3Aclient:se%2Bcr%25et+with+space")),
            Some(("my:client".to_string(), "se+cr%et with space".to_string()))
        );
    }

    #[test]
    fn plain_parts_are_unchanged_by_the_decoding() {
        assert_eq!(
            try_parse_basic_client_credentials(&encoded("orders-api:4c1f2d8e-9b7a")),
            Some(("orders-api".to_string(), "4c1f2d8e-9b7a".to_string()))
        );
    }

    #[test]
    fn the_scheme_name_is_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("basic Y2xpZW50OnNlY3JldA=="));
        assert_eq!(
            try_parse_basic_client_credentials(&headers),
            Some(("client".to_string(), "secret".to_string()))
        );
    }

    #[test]
    fn credentials_without_a_separator_are_refused() {
        assert_eq!(
            try_parse_basic_client_credentials(&encoded("client-only")),
            None
        );
    }

    #[test]
    fn rejects_malformed_base64() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Basic ???"));
        assert_eq!(try_parse_basic_client_credentials(&headers), None);
    }
}
