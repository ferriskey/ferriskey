use axum::http::HeaderMap;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ferriskey_api_core::api_entities::api_error::ApiError;
use ferriskey_core::domain::authentication::client_authentication::{
    ClientAssertion, ClientCredential, JWT_BEARER_ASSERTION_TYPE,
};

use crate::basic_auth::try_parse_basic_client_credentials;

pub const TOKEN_ENDPOINT_PATH: &str = "/protocol/openid-connect/token";

/// The client authentication parameters of a request body (RFC 6749 §2.3.1,
/// RFC 7521 §4.2).
#[derive(Debug, Default, Clone, Copy)]
pub struct ClientAuthParams<'a> {
    pub client_id: Option<&'a str>,
    pub client_secret: Option<&'a str>,
    pub client_assertion_type: Option<&'a str>,
    pub client_assertion: Option<&'a str>,
}

#[derive(Debug)]
pub struct PresentedClient {
    pub client_id: String,
    pub credential: ClientCredential,
}

/// The `aud` values an assertion sent to `endpoint_path` may carry: the realm
/// issuer, the token endpoint, and the endpoint itself.
pub fn assertion_audiences(issuer: &str, endpoint_path: &str) -> Vec<String> {
    let mut audiences = vec![issuer.to_string(), format!("{issuer}{TOKEN_ENDPOINT_PATH}")];
    if endpoint_path != TOKEN_ENDPOINT_PATH {
        audiences.push(format!("{issuer}{endpoint_path}"));
    }
    audiences
}

/// Reads who the client says it is and what it presented to prove it.
///
/// HTTP Basic wins over a secret in the body, as before. An assertion cannot
/// be combined with a secret (RFC 6749 §2.3). Without a `client_id`, an
/// assertion names its client in `sub` (RFC 7523 §3); the claim is read here
/// unverified, only to find the client whose keys will verify it.
pub fn presented_client(
    headers: &HeaderMap,
    params: ClientAuthParams<'_>,
    audiences: impl FnOnce() -> Vec<String>,
) -> Result<PresentedClient, ApiError> {
    let basic = try_parse_basic_client_credentials(headers);

    let Some(assertion) = params.client_assertion else {
        if params.client_assertion_type.is_some() {
            return Err(invalid_request("client_assertion is missing."));
        }
        return Ok(match basic {
            Some((client_id, secret)) => PresentedClient {
                client_id,
                credential: ClientCredential::secret(secret),
            },
            None => PresentedClient {
                client_id: params.client_id.unwrap_or_default().to_string(),
                credential: ClientCredential::from_secret(params.client_secret.map(str::to_string)),
            },
        });
    };

    if basic.is_some() || params.client_secret.is_some() {
        return Err(invalid_request(
            "Only one client authentication method may be used.",
        ));
    }

    if params.client_assertion_type != Some(JWT_BEARER_ASSERTION_TYPE) {
        return Err(invalid_client());
    }

    let client_id = match params.client_id {
        Some(client_id) => client_id.to_string(),
        None => assertion_subject(assertion).ok_or_else(invalid_client)?,
    };

    Ok(PresentedClient {
        client_id,
        credential: ClientCredential::Assertion(ClientAssertion::new(
            assertion.to_string(),
            audiences(),
        )),
    })
}

fn assertion_subject(assertion: &str) -> Option<String> {
    let payload = assertion.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    claims.get("sub")?.as_str().map(str::to_string)
}

fn invalid_request(description: &'static str) -> ApiError {
    ApiError::OAuthError {
        error: "invalid_request".into(),
        error_description: description.into(),
    }
}

fn invalid_client() -> ApiError {
    ApiError::OAuthUnauthorized {
        error: "invalid_client".into(),
        error_description: "Client authentication failed.".into(),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use ferriskey_core::domain::authentication::client_authentication::ClientCredential;

    use super::*;

    const ISSUER: &str = "https://iam.example/realms/acme";

    fn assertion_for(sub: &str) -> String {
        let payload = URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{sub}"}}"#));
        format!("eyJhbGciOiJSUzI1NiJ9.{payload}.c2ln")
    }

    fn basic() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Basic Y2xpZW50OnNlY3JldA=="),
        );
        headers
    }

    fn audiences() -> Vec<String> {
        assertion_audiences(ISSUER, TOKEN_ENDPOINT_PATH)
    }

    fn oauth_error(error: ApiError) -> String {
        match error {
            ApiError::OAuthError { error, .. } | ApiError::OAuthUnauthorized { error, .. } => {
                error.into_owned()
            }
            other => panic!("not an OAuth error: {other:?}"),
        }
    }

    #[test]
    fn an_assertion_names_its_client_in_sub() {
        let jwt = assertion_for("orders-api");
        let presented = presented_client(
            &HeaderMap::new(),
            ClientAuthParams {
                client_assertion_type: Some(JWT_BEARER_ASSERTION_TYPE),
                client_assertion: Some(&jwt),
                ..Default::default()
            },
            audiences,
        )
        .expect("presented");

        assert_eq!(presented.client_id, "orders-api");
        let ClientCredential::Assertion(assertion) = presented.credential else {
            panic!("expected an assertion");
        };
        assert_eq!(assertion.jwt.expose(), &jwt);
        assert_eq!(
            assertion.audiences,
            vec![
                ISSUER.to_string(),
                format!("{ISSUER}/protocol/openid-connect/token")
            ]
        );
    }

    #[test]
    fn an_explicit_client_id_is_kept_with_an_assertion() {
        let jwt = assertion_for("someone-else");
        let presented = presented_client(
            &HeaderMap::new(),
            ClientAuthParams {
                client_id: Some("orders-api"),
                client_assertion_type: Some(JWT_BEARER_ASSERTION_TYPE),
                client_assertion: Some(&jwt),
                ..Default::default()
            },
            audiences,
        )
        .expect("presented");

        assert_eq!(presented.client_id, "orders-api");
    }

    #[test]
    fn an_assertion_with_a_secret_is_refused() {
        let jwt = assertion_for("orders-api");
        let with_post = ClientAuthParams {
            client_secret: Some("s3cr3t"),
            client_assertion_type: Some(JWT_BEARER_ASSERTION_TYPE),
            client_assertion: Some(&jwt),
            ..Default::default()
        };
        let with_basic = ClientAuthParams {
            client_secret: None,
            ..with_post
        };

        for (headers, params) in [(HeaderMap::new(), with_post), (basic(), with_basic)] {
            let error = presented_client(&headers, params, audiences).expect_err("refused");
            assert_eq!(oauth_error(error), "invalid_request");
        }
    }

    #[test]
    fn another_assertion_type_is_invalid_client() {
        let jwt = assertion_for("orders-api");
        let error = presented_client(
            &HeaderMap::new(),
            ClientAuthParams {
                client_assertion_type: Some(
                    "urn:ietf:params:oauth:client-assertion-type:saml2-bearer",
                ),
                client_assertion: Some(&jwt),
                ..Default::default()
            },
            audiences,
        )
        .expect_err("refused");

        assert_eq!(oauth_error(error), "invalid_client");
    }

    #[test]
    fn an_unreadable_assertion_without_client_id_is_invalid_client() {
        let error = presented_client(
            &HeaderMap::new(),
            ClientAuthParams {
                client_assertion_type: Some(JWT_BEARER_ASSERTION_TYPE),
                client_assertion: Some("not-a-jwt"),
                ..Default::default()
            },
            audiences,
        )
        .expect_err("refused");

        assert_eq!(oauth_error(error), "invalid_client");
    }

    #[test]
    fn basic_still_wins_over_a_posted_secret() {
        let presented = presented_client(
            &basic(),
            ClientAuthParams {
                client_id: Some("other"),
                client_secret: Some("other-secret"),
                ..Default::default()
            },
            audiences,
        )
        .expect("presented");

        assert_eq!(presented.client_id, "client");
        let ClientCredential::Secret(secret) = presented.credential else {
            panic!("expected a secret");
        };
        assert_eq!(secret.expose(), "secret");
    }

    #[test]
    fn a_public_client_presents_nothing() {
        let presented = presented_client(
            &HeaderMap::new(),
            ClientAuthParams {
                client_id: Some("spa"),
                ..Default::default()
            },
            audiences,
        )
        .expect("presented");

        assert_eq!(presented.client_id, "spa");
        assert!(matches!(presented.credential, ClientCredential::None));
    }

    #[test]
    fn introspection_also_accepts_its_own_url_as_audience() {
        assert_eq!(
            assertion_audiences(ISSUER, "/protocol/openid-connect/token/introspect"),
            vec![
                ISSUER.to_string(),
                format!("{ISSUER}/protocol/openid-connect/token"),
                format!("{ISSUER}/protocol/openid-connect/token/introspect"),
            ]
        );
    }
}
