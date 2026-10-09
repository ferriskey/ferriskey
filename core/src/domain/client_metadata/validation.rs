use url::Url;

use crate::domain::client::entities::ClientRegistrationSource;
use crate::domain::client_metadata::entities::{
    ClientMetadataDocument, ClientMetadataError, MAX_CLIENT_ID_URL_LEN,
};
use crate::domain::client_registration::entities::MAX_REDIRECT_URIS;
use crate::domain::client_registration::validation::validate_redirect_uri;

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
        || url.path() == "/"
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

    if document.redirect_uris.len() > MAX_REDIRECT_URIS {
        return Err(ClientMetadataError::InvalidDocument(
            "redirect_uris holds too many uris",
        ));
    }

    if document
        .redirect_uris
        .iter()
        .any(|uri| validate_redirect_uri(uri).is_err())
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
    fn redirect_uris_follow_the_dynamic_registration_rules() {
        for uri in [
            "http://app.example/cb",
            "https://user:pw@app.example/cb",
            "ftp://app.example/cb",
            "myapp://callback",
            &format!("https://app.example/{}", "a".repeat(2100)),
        ] {
            let mut doc = document();
            doc.redirect_uris.push(uri.to_string());
            assert!(
                matches!(
                    validate_document(URL, &doc, CALLBACK),
                    Err(ClientMetadataError::InvalidDocument(_))
                ),
                "{uri}"
            );
        }
    }

    #[test]
    fn loopback_http_redirect_uris_are_accepted() {
        let mut doc = document();
        doc.redirect_uris
            .push("http://127.0.0.1:8080/cb".to_string());
        doc.redirect_uris.push("http://localhost/cb".to_string());
        assert!(validate_document(URL, &doc, "http://localhost/cb").is_ok());
    }

    #[test]
    fn a_document_with_too_many_redirect_uris_is_refused() {
        let mut doc = document();
        doc.redirect_uris = (0..=MAX_REDIRECT_URIS)
            .map(|index| format!("https://app.example/cb{index}"))
            .collect();
        assert!(matches!(
            validate_document(URL, &doc, "https://app.example/cb0"),
            Err(ClientMetadataError::InvalidDocument(_))
        ));

        doc.redirect_uris.truncate(MAX_REDIRECT_URIS);
        assert!(validate_document(URL, &doc, "https://app.example/cb0").is_ok());
    }

    #[test]
    fn a_client_id_url_needs_a_path() {
        for raw in ["https://app.example/", "https://app.example"] {
            assert_eq!(
                parse_client_id_url(raw, false),
                Err(ClientMetadataError::InvalidUrl),
                "{raw}"
            );
        }
        assert!(parse_client_id_url("https://app.example/client.json", false).is_ok());
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
