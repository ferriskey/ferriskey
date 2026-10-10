use std::collections::HashSet;

use url::{Host, Url};

use crate::domain::client_registration::entities::{
    AUTH_METHOD_BASIC, AUTH_METHOD_NONE, AUTH_METHOD_POST, ClientRegistrationRequest,
    DEFAULT_CLIENT_NAME, GRANT_AUTHORIZATION_CODE, GRANT_REFRESH_TOKEN, MAX_CLIENT_NAME_CHARS,
    MAX_REDIRECT_URI_BYTES, MAX_REDIRECT_URIS, RESPONSE_TYPE_CODE, RegistrationError,
    ValidatedRegistration,
};
use crate::domain::client_registration::provision::STANDARD_SCOPES;

pub fn validate_redirect_uri(raw: &str) -> Result<(), RegistrationError> {
    if raw.len() > MAX_REDIRECT_URI_BYTES {
        return Err(RegistrationError::InvalidRedirectUri(
            "redirect uri too long",
        ));
    }

    let url = Url::parse(raw)
        .map_err(|_| RegistrationError::InvalidRedirectUri("redirect uri is not a valid url"))?;

    if url.fragment().is_some() || !url.username().is_empty() || url.password().is_some() {
        return Err(RegistrationError::InvalidRedirectUri(
            "redirect uri must not carry a fragment or credentials",
        ));
    }

    match (url.scheme(), url.host()) {
        ("https", Some(_)) => Ok(()),
        ("http", Some(host)) if is_loopback_host(&host) => Ok(()),
        _ => Err(RegistrationError::InvalidRedirectUri(
            "redirect uri must be https, or http on a loopback host",
        )),
    }
}

fn is_loopback_host(host: &Host<&str>) -> bool {
    match host {
        Host::Domain(domain) => domain.eq_ignore_ascii_case("localhost"),
        Host::Ipv4(ip) => ip.is_loopback(),
        Host::Ipv6(ip) => ip.is_loopback(),
    }
}

pub fn validate_registration(
    request: ClientRegistrationRequest,
    registrable_scopes: &HashSet<String>,
) -> Result<ValidatedRegistration, RegistrationError> {
    if request.redirect_uris.is_empty() || request.redirect_uris.len() > MAX_REDIRECT_URIS {
        return Err(RegistrationError::InvalidRedirectUri(
            "redirect_uris must hold between 1 and 10 uris",
        ));
    }

    for uri in &request.redirect_uris {
        validate_redirect_uri(uri)?;
    }

    let token_endpoint_auth_method = request
        .token_endpoint_auth_method
        .unwrap_or_else(|| AUTH_METHOD_BASIC.to_string());

    if ![AUTH_METHOD_NONE, AUTH_METHOD_BASIC, AUTH_METHOD_POST]
        .contains(&token_endpoint_auth_method.as_str())
    {
        return Err(RegistrationError::InvalidClientMetadata(
            "unsupported token_endpoint_auth_method",
        ));
    }

    let grant_types = request
        .grant_types
        .unwrap_or_else(|| vec![GRANT_AUTHORIZATION_CODE.to_string()]);

    if !grant_types
        .iter()
        .all(|grant| [GRANT_AUTHORIZATION_CODE, GRANT_REFRESH_TOKEN].contains(&grant.as_str()))
        || !grant_types
            .iter()
            .any(|grant| grant == GRANT_AUTHORIZATION_CODE)
    {
        return Err(RegistrationError::InvalidClientMetadata(
            "grant_types must include authorization_code and may only add refresh_token",
        ));
    }

    let response_types = request
        .response_types
        .unwrap_or_else(|| vec![RESPONSE_TYPE_CODE.to_string()]);

    if response_types.is_empty() || !response_types.iter().all(|kind| kind == RESPONSE_TYPE_CODE) {
        return Err(RegistrationError::InvalidClientMetadata(
            "response_types may only be code",
        ));
    }

    let scopes = match request.scope.as_deref() {
        None => None,
        Some(raw) => {
            let requested: Vec<String> = raw.split_whitespace().map(str::to_string).collect();
            if !requested
                .iter()
                .all(|scope| registrable_scopes.contains(scope))
            {
                return Err(RegistrationError::InvalidClientMetadata(
                    "scope is not available for dynamic clients in this realm",
                ));
            }
            Some(requested)
        }
    };

    let client_name = match request.client_name.as_deref().map(str::trim) {
        Some(name) if name.chars().count() > MAX_CLIENT_NAME_CHARS => {
            return Err(RegistrationError::InvalidClientMetadata(
                "client_name is too long",
            ));
        }
        Some(name) if !name.is_empty() => name.to_string(),
        _ => DEFAULT_CLIENT_NAME.to_string(),
    };

    Ok(ValidatedRegistration {
        client_name,
        redirect_uris: request.redirect_uris,
        public_client: token_endpoint_auth_method == AUTH_METHOD_NONE,
        token_endpoint_auth_method,
        grant_types,
        response_types,
        scopes,
    })
}

pub fn effective_scope(requested: &[String]) -> String {
    let mut granted: Vec<&str> = STANDARD_SCOPES.to_vec();
    for scope in requested {
        if !granted.contains(&scope.as_str()) {
            granted.push(scope);
        }
    }
    granted.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scopes() -> HashSet<String> {
        STANDARD_SCOPES.iter().map(|s| s.to_string()).collect()
    }

    fn request() -> ClientRegistrationRequest {
        ClientRegistrationRequest {
            redirect_uris: vec!["https://app.example/callback".to_string()],
            client_name: Some("App".to_string()),
            token_endpoint_auth_method: Some("none".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn https_and_loopback_http_redirect_uris_are_accepted() {
        for uri in [
            "https://app.example/cb",
            "http://localhost:8080/cb",
            "http://127.0.0.1/cb",
            "http://[::1]:9000/cb",
        ] {
            assert!(validate_redirect_uri(uri).is_ok(), "{uri}");
        }
    }

    #[test]
    fn other_redirect_uris_are_refused() {
        for uri in [
            "http://app.example/cb",
            "http://localhost.evil.example/cb",
            "ftp://app.example/cb",
            "myapp://callback",
            "javascript:alert(1)",
            "^.*$",
            "https://app.example/cb#frag",
            "https://user:pw@app.example/cb",
            "not a url",
        ] {
            assert!(
                matches!(
                    validate_redirect_uri(uri),
                    Err(RegistrationError::InvalidRedirectUri(_))
                ),
                "{uri}"
            );
        }
    }

    #[test]
    fn a_public_registration_is_accepted() {
        let validated = validate_registration(request(), &scopes()).expect("valid");

        assert!(validated.public_client);
        assert_eq!(validated.grant_types, vec!["authorization_code"]);
        assert_eq!(validated.response_types, vec!["code"]);
        assert_eq!(validated.client_name, "App");
    }

    #[test]
    fn the_default_auth_method_is_a_confidential_one() {
        let mut req = request();
        req.token_endpoint_auth_method = None;

        let validated = validate_registration(req, &scopes()).expect("valid");

        assert!(!validated.public_client);
        assert_eq!(validated.token_endpoint_auth_method, "client_secret_basic");
    }

    #[test]
    fn redirect_uris_are_required() {
        let mut req = request();
        req.redirect_uris.clear();

        assert!(matches!(
            validate_registration(req, &scopes()),
            Err(RegistrationError::InvalidRedirectUri(_))
        ));
    }

    #[test]
    fn unsupported_metadata_is_refused() {
        let cases: Vec<fn(&mut ClientRegistrationRequest)> = vec![
            |r| r.token_endpoint_auth_method = Some("private_key_jwt".to_string()),
            |r| r.grant_types = Some(vec!["password".to_string()]),
            |r| r.grant_types = Some(vec!["refresh_token".to_string()]),
            |r| r.grant_types = Some(vec!["client_credentials".to_string()]),
            |r| r.response_types = Some(vec!["token".to_string()]),
            |r| r.response_types = Some(vec![]),
            |r| r.scope = Some("openid admin".to_string()),
            |r| r.client_name = Some("x".repeat(129)),
        ];

        for mutate in cases {
            let mut req = request();
            mutate(&mut req);
            assert!(matches!(
                validate_registration(req, &scopes()),
                Err(RegistrationError::InvalidClientMetadata(_))
            ));
        }
    }

    #[test]
    fn refresh_token_may_accompany_authorization_code() {
        let mut req = request();
        req.grant_types = Some(vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ]);

        assert!(validate_registration(req, &scopes()).is_ok());
    }

    #[test]
    fn a_missing_name_gets_a_default() {
        let mut req = request();
        req.client_name = None;

        let validated = validate_registration(req, &scopes()).expect("valid");

        assert_eq!(validated.client_name, DEFAULT_CLIENT_NAME);
    }

    #[test]
    fn the_effective_scope_always_lists_the_standard_scopes_once() {
        assert_eq!(
            effective_scope(&["openid".to_string(), "extra".to_string()]),
            "openid profile email roles extra"
        );
    }
}
