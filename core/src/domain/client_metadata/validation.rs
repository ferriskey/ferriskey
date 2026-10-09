use url::Url;

use crate::domain::client::entities::ClientRegistrationSource;
use crate::domain::client_metadata::entities::{
    ClientMetadataDocument, ClientMetadataError, MAX_CLIENT_ID_URL_LEN,
};

const FORBIDDEN_REDIRECT_SCHEMES: [&str; 3] = ["javascript", "data", "vbscript"];

pub fn is_metadata_client_id(client_id: &str, allow_cleartext: bool) -> bool {
    client_id.starts_with("https://") || (allow_cleartext && client_id.starts_with("http://"))
}

pub fn parse_client_id_url(raw: &str, allow_cleartext: bool) -> Result<Url, ClientMetadataError> {
    if raw.len() > MAX_CLIENT_ID_URL_LEN {
        return Err(ClientMetadataError::InvalidUrl);
    }

    let url = Url::parse(raw).map_err(|_| ClientMetadataError::InvalidUrl)?;

    let scheme_allowed = url.scheme() == "https" || (allow_cleartext && url.scheme() == "http");

    if !scheme_allowed
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.as_str() != raw
    {
        return Err(ClientMetadataError::InvalidUrl);
    }

    Ok(url)
}

pub fn ensure_host_allowed(
    host: &str,
    allowed_hosts: &[String],
) -> Result<(), ClientMetadataError> {
    if allowed_hosts.is_empty()
        || allowed_hosts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(host))
    {
        return Ok(());
    }

    Err(ClientMetadataError::HostNotAllowed)
}

pub fn validate_document(
    client_id_url: &str,
    document: &ClientMetadataDocument,
    redirect_uri: &str,
) -> Result<(), ClientMetadataError> {
    if document.client_id != client_id_url {
        return Err(ClientMetadataError::ClientIdMismatch);
    }

    match document.token_endpoint_auth_method.as_deref() {
        None | Some("none") => {}
        Some(_) => return Err(ClientMetadataError::UnsupportedAuthMethod),
    }

    if document.redirect_uris.is_empty() {
        return Err(ClientMetadataError::InvalidDocument(
            "redirect_uris is empty",
        ));
    }

    if !document
        .redirect_uris
        .iter()
        .all(|uri| is_acceptable_redirect_uri(uri))
    {
        return Err(ClientMetadataError::InvalidDocument(
            "redirect_uris holds an unusable uri",
        ));
    }

    if !document.redirect_uris.iter().any(|uri| uri == redirect_uri) {
        return Err(ClientMetadataError::RedirectUriNotListed);
    }

    Ok(())
}

fn is_acceptable_redirect_uri(raw: &str) -> bool {
    match Url::parse(raw) {
        Ok(url) => url.fragment().is_none() && !FORBIDDEN_REDIRECT_SCHEMES.contains(&url.scheme()),
        Err(_) => false,
    }
}

pub fn client_uri_host(
    source: ClientRegistrationSource,
    client_id: &str,
    first_redirect_uri: Option<&str>,
) -> Option<String> {
    let origin = match source {
        ClientRegistrationSource::Admin => return None,
        ClientRegistrationSource::MetadataDocument => client_id,
        ClientRegistrationSource::Dynamic => first_redirect_uri?,
    };

    Url::parse(origin)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://app.example/client.json";
    const CALLBACK: &str = "https://app.example/callback";

    fn document() -> ClientMetadataDocument {
        ClientMetadataDocument {
            client_id: URL.to_string(),
            client_name: Some("App".to_string()),
            redirect_uris: vec![CALLBACK.to_string()],
            token_endpoint_auth_method: Some("none".to_string()),
        }
    }

    #[test]
    fn only_https_urls_are_metadata_client_ids_unless_cleartext_is_allowed() {
        assert!(is_metadata_client_id(URL, false));
        assert!(!is_metadata_client_id("http://app.example/c.json", false));
        assert!(is_metadata_client_id("http://app.example/c.json", true));
        assert!(!is_metadata_client_id("my-client", true));
    }

    #[test]
    fn a_well_formed_https_url_parses() {
        assert!(parse_client_id_url(URL, false).is_ok());
    }

    #[test]
    fn cleartext_urls_parse_only_in_test_mode() {
        assert_eq!(
            parse_client_id_url("http://127.0.0.1:9/c.json", false),
            Err(ClientMetadataError::InvalidUrl)
        );
        assert!(parse_client_id_url("http://127.0.0.1:9/c.json", true).is_ok());
    }

    #[test]
    fn urls_longer_than_the_column_are_refused() {
        let long = format!("https://app.example/{}", "a".repeat(250));
        assert_eq!(
            parse_client_id_url(&long, false),
            Err(ClientMetadataError::InvalidUrl)
        );
    }

    #[test]
    fn urls_with_credentials_fragments_or_a_non_canonical_form_are_refused() {
        for raw in [
            "https://user:pw@app.example/c.json",
            "https://app.example/c.json#frag",
            "https://APP.example/c.json",
            "https://app.example/a/../c.json",
            "ftp://app.example/c.json",
            "not a url",
        ] {
            assert_eq!(
                parse_client_id_url(raw, false),
                Err(ClientMetadataError::InvalidUrl),
                "{raw}"
            );
        }
    }

    #[test]
    fn an_empty_allowlist_accepts_any_host() {
        assert!(ensure_host_allowed("app.example", &[]).is_ok());
    }

    #[test]
    fn the_allowlist_compares_hosts_case_insensitively() {
        let allowed = vec!["App.Example".to_string()];
        assert!(ensure_host_allowed("app.example", &allowed).is_ok());
        assert_eq!(
            ensure_host_allowed("evil.example", &allowed),
            Err(ClientMetadataError::HostNotAllowed)
        );
    }

    #[test]
    fn a_matching_document_is_accepted() {
        assert!(validate_document(URL, &document(), CALLBACK).is_ok());
    }

    #[test]
    fn an_absent_auth_method_is_treated_as_none() {
        let mut doc = document();
        doc.token_endpoint_auth_method = None;
        assert!(validate_document(URL, &doc, CALLBACK).is_ok());
    }

    #[test]
    fn a_client_id_that_differs_from_the_url_is_refused() {
        let mut doc = document();
        doc.client_id = "https://other.example/client.json".to_string();
        assert_eq!(
            validate_document(URL, &doc, CALLBACK),
            Err(ClientMetadataError::ClientIdMismatch)
        );
    }

    #[test]
    fn private_key_jwt_and_secret_methods_are_refused() {
        for method in [
            "private_key_jwt",
            "client_secret_basic",
            "client_secret_post",
        ] {
            let mut doc = document();
            doc.token_endpoint_auth_method = Some(method.to_string());
            assert_eq!(
                validate_document(URL, &doc, CALLBACK),
                Err(ClientMetadataError::UnsupportedAuthMethod)
            );
        }
    }

    #[test]
    fn a_document_without_redirect_uris_is_refused() {
        let mut doc = document();
        doc.redirect_uris.clear();
        assert!(matches!(
            validate_document(URL, &doc, CALLBACK),
            Err(ClientMetadataError::InvalidDocument(_))
        ));
    }

    #[test]
    fn a_redirect_uri_that_is_not_listed_is_refused() {
        assert_eq!(
            validate_document(URL, &document(), "https://app.example/other"),
            Err(ClientMetadataError::RedirectUriNotListed)
        );
    }

    #[test]
    fn the_redirect_uri_comparison_is_exact() {
        assert_eq!(
            validate_document(URL, &document(), "https://app.example/callback/"),
            Err(ClientMetadataError::RedirectUriNotListed)
        );
    }

    #[test]
    fn pattern_and_script_redirect_uris_in_a_document_are_refused() {
        for uri in ["^.*$", "javascript:alert(1)", "https://app.example/cb#frag"] {
            let mut doc = document();
            doc.redirect_uris.push(uri.to_string());
            assert!(matches!(
                validate_document(URL, &doc, CALLBACK),
                Err(ClientMetadataError::InvalidDocument(_))
            ));
        }
    }

    #[test]
    fn the_consent_host_follows_the_registration_source() {
        assert_eq!(
            client_uri_host(ClientRegistrationSource::MetadataDocument, URL, None),
            Some("app.example".to_string())
        );
        assert_eq!(
            client_uri_host(
                ClientRegistrationSource::Dynamic,
                "uuid",
                Some("https://cb.example/x")
            ),
            Some("cb.example".to_string())
        );
        assert_eq!(
            client_uri_host(ClientRegistrationSource::Dynamic, "uuid", None),
            None
        );
        assert_eq!(
            client_uri_host(ClientRegistrationSource::Admin, URL, Some(CALLBACK)),
            None
        );
    }
}
